#!/usr/bin/env bash
# JobFlow macOS 打包脚本
#
# 用法：
#   ./scripts/package-macos.sh                    # release 构建 + JobFlow.app + zip (+ dmg)
#   ./scripts/package-macos.sh --skip-build       # 只重新打包
#   ./scripts/package-macos.sh --no-dmg           # 不生成 dmg
#   ./scripts/package-macos.sh --no-default-features
#   ./scripts/package-macos.sh --features <features>
#   ./scripts/package-macos.sh --sign "Developer ID Application: Your Name (TEAMID)"
#
# 产物：
#   dist/JobFlow.app
#   dist/job-tracker-<version>-macos-<arch>.zip
#   dist/job-tracker-<version>-macos-<arch>.dmg        （如果系统有 hdiutil）
#   dist/*.sha256

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(grep -E '^version[[:space:]]*=' Cargo.toml | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/')"
if [[ -z "${VERSION}" ]]; then
    echo "无法从 Cargo.toml 读取版本号" >&2
    exit 1
fi

APP_NAME="JobFlow"
BUNDLE_ID="com.microfatrat.jobtracker"
EXECUTABLE="job-tracker"
TARGET_DIR="${CARGO_TARGET_DIR:-target}"
DIST="$ROOT/dist"
SKIP_BUILD=0
BUILD_DMG=1
SIGN_IDENTITY="${MACOS_SIGN_IDENTITY:-}"
CARGO_ARGS=(--release)

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-build)
            SKIP_BUILD=1
            shift
            ;;
        --no-dmg)
            BUILD_DMG=0
            shift
            ;;
        --no-default-features)
            CARGO_ARGS+=(--no-default-features)
            shift
            ;;
        --features)
            CARGO_ARGS+=(--features "$2")
            shift 2
            ;;
        --sign)
            SIGN_IDENTITY="$2"
            shift 2
            ;;
        *)
            CARGO_ARGS+=("$1")
            shift
            ;;
    esac
done

case "$(uname -m)" in
    arm64 | aarch64)
        ARCH="arm64"
        ;;
    x86_64 | amd64)
        ARCH="x86_64"
        ;;
    *)
        ARCH="$(uname -m)"
        ;;
esac

sha256() {
    if command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1"
    else
        sha256sum "$1"
    fi
}

BIN="$TARGET_DIR/release/job-tracker"
STAGE_ROOT="$DIST/.stage"
APP="$STAGE_ROOT/$APP_NAME.app"
ZIP="$DIST/job-tracker-${VERSION}-macos-$ARCH.zip"
DMG="$DIST/job-tracker-${VERSION}-macos-$ARCH.dmg"

echo "JobFlow macOS 打包"
echo "  版本号: ${VERSION}"
echo "  架构:   $ARCH"
echo "  输出:   $DIST"

if [[ $SKIP_BUILD -eq 0 ]]; then
    echo ""
    echo "[1/5] cargo build ${CARGO_ARGS[*]}"
    cargo build "${CARGO_ARGS[@]}"
else
    echo ""
    echo "[1/5] 跳过构建（--skip-build）"
fi

if [[ ! -x "$BIN" ]]; then
    echo "找不到可执行文件 $BIN，请先执行 cargo build --release" >&2
    exit 1
fi

echo ""
echo "[2/5] 生成 $APP_NAME.app"
rm -rf "$STAGE_ROOT"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/$EXECUTABLE"
chmod 755 "$APP/Contents/MacOS/$EXECUTABLE"
cp README.md LICENSE "$APP/Contents/Resources/"
sed -e "s/@APP_NAME@/$APP_NAME/g" \
    -e "s/@BUNDLE_ID@/$BUNDLE_ID/g" \
    -e "s/@VERSION@/${VERSION}/g" \
    -e "s/@EXECUTABLE@/$EXECUTABLE/g" \
    packaging/macos/Info.plist > "$APP/Contents/Info.plist"

echo ""
echo "[3/5] 代码签名"
if command -v codesign >/dev/null 2>&1; then
    if [[ -n "$SIGN_IDENTITY" ]]; then
        codesign --force --deep --options runtime --sign "$SIGN_IDENTITY" "$APP"
        echo "  已使用正式证书签名: $SIGN_IDENTITY"
    else
        codesign --force --deep --sign - "$APP"
        echo "  已使用 ad-hoc 签名（本地运行可用；发布需要 Developer ID）"
    fi
else
    echo "  未找到 codesign，跳过签名"
fi

echo ""
echo "[4/5] 生成 zip"
mkdir -p "$DIST"
rm -f "$ZIP"
(
    cd "$STAGE_ROOT"
    zip -qry "$ZIP" "$APP_NAME.app"
)
sha256 "$ZIP" > "$ZIP.sha256"
echo "  $ZIP"

if [[ $BUILD_DMG -eq 1 ]] && command -v hdiutil >/dev/null 2>&1; then
    echo ""
    echo "[5/5] 生成 dmg"
    DMG_STAGE="$STAGE_ROOT/dmg"
    rm -rf "$DMG_STAGE"
    mkdir -p "$DMG_STAGE"
    cp -R "$APP" "$DMG_STAGE/$APP_NAME.app"
    ln -s /Applications "$DMG_STAGE/Applications"
    rm -f "$DMG"
    hdiutil create -volname "$APP_NAME" -srcfolder "$DMG_STAGE" -ov -format UDZO "$DMG" >/dev/null
    sha256 "$DMG" > "$DMG.sha256"
    echo "  $DMG"
else
    echo ""
    echo "[5/5] 跳过 dmg（没有 hdiutil 或使用了 --no-dmg）"
fi

# 把 .app 复制到 dist 根目录，方便直接拖拽安装
rm -rf "$DIST/$APP_NAME.app"
cp -R "$APP" "$DIST/$APP_NAME.app"
rm -rf "$STAGE_ROOT"

echo ""
echo "打包完成："
ls -lh "$DIST" | grep -E "job-tracker.*(zip|dmg|sha256)|$APP_NAME.app" || true
