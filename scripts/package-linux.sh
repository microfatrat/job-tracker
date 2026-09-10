#!/usr/bin/env bash
# JobFlow Linux 打包脚本
#
# 用法：
#   ./scripts/package-linux.sh                    # release 构建 + tar.gz
#   ./scripts/package-linux.sh --deb              # 额外生成 .deb
#   ./scripts/package-linux.sh --skip-build       # 只重新打包
#   ./scripts/package-linux.sh --no-default-features
#   ./scripts/package-linux.sh --features windows-gui

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERSION="$(grep -E '^version[[:space:]]*=' Cargo.toml | head -n 1 | sed -E 's/.*"([^"]+)".*/\1/')"
if [[ -z "${VERSION}" ]]; then
    echo "无法从 Cargo.toml 读取版本号" >&2
    exit 1
fi

TARGET_DIR="${CARGO_TARGET_DIR:-target}"
DIST="$ROOT/dist"
SKIP_BUILD=0
BUILD_DEB=0
CARGO_ARGS=(--release)

while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-build)
            SKIP_BUILD=1
            shift
            ;;
        --deb)
            BUILD_DEB=1
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
        *)
            CARGO_ARGS+=("$1")
            shift
            ;;
    esac
done

case "$(uname -m)" in
    x86_64 | amd64)
        ARCH="x86_64"
        DEB_ARCH="amd64"
        ;;
    aarch64 | arm64)
        ARCH="aarch64"
        DEB_ARCH="arm64"
        ;;
    *)
        ARCH="$(uname -m)"
        DEB_ARCH="$(uname -m)"
        ;;
esac

BIN="$TARGET_DIR/release/job-tracker"
STAGE="$DIST/.stage/job-tracker-${VERSION}-linux-$ARCH"
TARBALL="$DIST/job-tracker-${VERSION}-linux-$ARCH.tar.gz"

echo "JobFlow Linux 打包"
echo "  版本号: ${VERSION}"
echo "  架构:   $ARCH"
echo "  输出:   $DIST"

if [[ $SKIP_BUILD -eq 0 ]]; then
    echo ""
    echo "[1/4] cargo build ${CARGO_ARGS[*]}"
    cargo build "${CARGO_ARGS[@]}"
else
    echo ""
    echo "[1/4] 跳过构建（--skip-build）"
fi

if [[ ! -x "$BIN" ]]; then
    echo "找不到可执行文件 $BIN，请先执行 cargo build --release" >&2
    exit 1
fi

echo ""
echo "[2/4] 准备发布目录"
rm -rf "$DIST/.stage"
mkdir -p "$STAGE"
cp "$BIN" "$STAGE/job-tracker"
cp README.md "$STAGE/README.md"
cp LICENSE "$STAGE/LICENSE"
cp packaging/linux/job-tracker.desktop "$STAGE/job-tracker.desktop"
chmod 755 "$STAGE/job-tracker"

echo ""
echo "[3/4] 生成 tar.gz"
mkdir -p "$DIST"
tar -czf "$TARBALL" -C "$STAGE" .
sha256sum "$TARBALL" > "$TARBALL.sha256"
rm -rf "$DIST/.stage"
echo "  $TARBALL"

if [[ $BUILD_DEB -eq 1 ]]; then
    if ! command -v dpkg-deb >/dev/null 2>&1; then
        echo "未找到 dpkg-deb，无法生成 .deb；请安装 dpkg-dev 或使用 tar.gz" >&2
        exit 1
    fi
    echo ""
    echo "[4/4] 生成 .deb"
    DEB_ROOT="$DIST/deb-stage"
    rm -rf "$DEB_ROOT"
    mkdir -p "$DEB_ROOT/DEBIAN"
    mkdir -p "$DEB_ROOT/usr/bin"
    mkdir -p "$DEB_ROOT/usr/share/doc/job-tracker"
    mkdir -p "$DEB_ROOT/usr/share/applications"
    cp "$BIN" "$DEB_ROOT/usr/bin/job-tracker"
    chmod 755 "$DEB_ROOT/usr/bin/job-tracker"
    cp README.md "$DEB_ROOT/usr/share/doc/job-tracker/README.md"
    cp LICENSE "$DEB_ROOT/usr/share/doc/job-tracker/LICENSE"
    cp packaging/linux/job-tracker.desktop "$DEB_ROOT/usr/share/applications/job-tracker.desktop"
    cat > "$DEB_ROOT/DEBIAN/control" <<EOF
Package: job-tracker
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${DEB_ARCH}
Depends: libxkbcommon0, libxkbcommon-x11-0, libvulkan1, libfontconfig1, libfreetype6, libwayland-client0, libxcb1, libx11-xcb1, libxshmfence1
Maintainer: JobFlow contributors <noreply@example.com>
Homepage: https://github.com/your-name/job-tracker
Description: 求职简历投递与面试阶段统计系统
 基于 Rust + GPUI 的桌面应用，用于管理投递记录、跟踪面试阶段并统计漏斗转化率。
EOF
    dpkg-deb --build --root-owner-group "$DEB_ROOT" "$DIST/job-tracker_${VERSION}_${DEB_ARCH}.deb"
    rm -rf "$DEB_ROOT"
    echo "  $DIST/job-tracker_${VERSION}_${DEB_ARCH}.deb"
else
    echo ""
    echo "[4/4] 跳过 .deb（加 --deb 可生成）"
fi

echo ""
echo "打包完成："
ls -lh "$DIST" | grep -E "job-tracker.*(tar.gz|deb|sha256)" || true
