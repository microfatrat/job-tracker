# 打包与发布指南

本文档说明如何把 JobFlow 打包成可分发的 Windows / Linux 安装包或压缩包，以及如何通过 GitHub Actions 自动发布。

---

## 0. 发布前检查

1. 更新版本号（见文末“版本号管理”）。
2. 运行格式化、静态检查和测试：

```bash
cargo fmt --check
cargo check --all-targets
cargo test --no-default-features
```

3. 确认演示数据文件没有被提交（默认数据文件在用户目录，不在仓库里）。
4. 如果本次改动涉及 Windows 打包，确认 Windows CI 通过。

---

## 1. 通用 release 构建

```bash
cargo build --release
```

产物位置：

- Windows：`target/release/job-tracker.exe`
- Linux/macOS：`target/release/job-tracker`

release profile 已经配置了：

```toml
[profile.release]
opt-level = 2
lto = "thin"
codegen-units = 1
strip = "symbols"
```

---

## 2. Windows 打包

### 2.1 便携版 ZIP（推荐先做这个）

在 PowerShell 中执行：

```powershell
.\scripts\package-windows.ps1
```

脚本会：

1. 自动搜索 Windows SDK 中的 `fxc.exe` 并设置 `GPUI_FXC_PATH`（release 构建需要它；找不到时会给出明确提示）；
2. 执行 `cargo build --release`；
3. 在 `dist/` 下生成类似 `job-tracker-0.1.0-windows-x86_64.zip` 的压缩包；
4. 压缩包内包含 `job-tracker.exe`、`README.md`、`LICENSE`。

> 如果只想做 debug 版本，可以直接 `cargo build`，产物在 `target/debug/job-tracker.exe`；debug 构建不需要 `fxc.exe`。

常用参数：

```powershell
.\scripts\package-windows.ps1 -NoDefaultFeatures   # 跳过 windows-manifest（没有 rc.exe 时）
.\scripts\package-windows.ps1 -SkipBuild           # 只重新打包，不重新编译
.\scripts\package-windows.ps1 -Installer           # 同时用 Inno Setup 生成安装程序
```

### 2.2 无控制台窗口的发布版

如果希望双击 exe 时不弹出控制台窗口：

```powershell
cargo build --release --features windows-gui
```

或者在打包脚本中使用：

```powershell
.\scripts\package-windows.ps1 -WindowsGui
```

注意：启用 `windows-gui` 后，release 版的 `--report` / `--seed` 控制台输出不可见；需要命令行输出时请使用 debug 构建。

### 2.3 安装程序（Inno Setup）

1. 安装 [Inno Setup](https://jrsoftware.org/isinfo.php)；
2. 执行：

```powershell
.\scripts\package-windows.ps1 -Installer
```

或者手动执行：

```powershell
iscc /DMyAppVersion=0.1.0 packaging\windows\job-tracker.iss
```

脚本模板在 `packaging/windows/job-tracker.iss`，默认安装到 `Program Files\JobFlow`，并创建开始菜单和桌面快捷方式。

### 2.4 VC++ 运行库

Rust MSVC 默认动态链接 VC 运行库（`VCRUNTIME140.dll` 等）。Windows 10/11 通常已自带；如果目标机器缺少，可以：

- 在安装包中附带并安装 [Microsoft Visual C++ Redistributable](https://aka.ms/vs/17/release/vc_redist.x64.exe)；或
- 使用静态 CRT 构建（在 `.cargo/config.toml` 中添加）：

```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

静态 CRT 会让 exe 变大，但不再依赖 VC++ 运行库。

### 2.5 代码签名（可选）

如果已有代码签名证书，可以用 `signtool`：

```powershell
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /a target\release\job-tracker.exe
```

签名后再执行打包脚本（`-SkipBuild`），确保压缩包/安装包里的 exe 已签名。

---

## 3. Linux 打包

### 3.1 tar.gz 压缩包

```bash
./scripts/package-linux.sh
```

产物：`dist/job-tracker-0.1.0-linux-x86_64.tar.gz`，内含二进制、`README.md`、`LICENSE`。

如果机器没有图形依赖，可以只做无头版本：

```bash
./scripts/package-linux.sh --no-default-features
```

### 3.2 Debian / Ubuntu .deb

```bash
./scripts/package-linux.sh --deb
```

脚本使用 `dpkg-deb` 生成 `dist/job-tracker_0.1.0_amd64.deb`，并在 control 文件中声明运行依赖：

```text
libxkbcommon0, libxkbcommon-x11-0, libvulkan1,
libfontconfig1, libfreetype6, libwayland-client0,
libxcb1, libx11-xcb1, libxshmfence1
```

安装：

```bash
sudo dpkg -i dist/job-tracker_0.1.0_amd64.deb
sudo apt-get -f install   # 如果缺少依赖
```

### 3.3 Fedora / RHEL .rpm

可以使用 `cargo-generate-rpm`：

```bash
cargo install cargo-generate-rpm
cargo build --release
cargo generate-rpm
```

默认产物在 `target/generate-rpm/` 下。也可以把二进制和 `README.md` 放进自己的 `rpmbuild` 目录手工打包。

### 3.4 AppImage

AppImage 对系统 Vulkan 驱动、fontconfig 和字体有依赖，GPUI 应用不一定适合做成完全自包含的 AppImage。更推荐用 `.deb` / `.rpm` 或在目标发行版上编译。

如果确实需要 AppImage，可以使用 `linuxdeploy`，并把 Vulkan 驱动作为运行时依赖处理。

---

## 4. macOS 打包

前提：安装 Xcode Command Line Tools：

```bash
xcode-select --install
```

一键打包：

```bash
./scripts/package-macos.sh
```

脚本会：

1. 执行 `cargo build --release`；
2. 生成 `dist/JobFlow.app`（包含 `Contents/MacOS/job-tracker` 和 `Contents/Info.plist`）；
3. 生成 `dist/job-tracker-<version>-macos-<arch>.zip`；
4. 如果系统有 `hdiutil`，还会生成 `.dmg`（内含 Applications 快捷方式）；
5. 生成对应的 SHA256 校验和；
6. 如果找到 `codesign`，默认做 ad-hoc 签名，方便本机运行。

常用参数：

```bash
./scripts/package-macos.sh --skip-build
./scripts/package-macos.sh --no-dmg
./scripts/package-macos.sh --no-default-features
./scripts/package-macos.sh --sign "Developer ID Application: Your Name (TEAMID)"
```

架构：在 Apple Silicon 上生成 `arm64` 包，在 Intel Mac 上生成 `x86_64` 包。

正式发布：

1. 使用 Developer ID 证书签名：

   ```bash
   ./scripts/package-macos.sh --sign "Developer ID Application: Your Name (TEAMID)"
   ```

2. 使用 `notarytool` 提交公证：

   ```bash
   xcrun notarytool submit dist/job-tracker-<version>-macos-<arch>.dmg \
     --apple-id "you@example.com" --team-id TEAMID --password "app-specific-password" --wait
   ```

3. 装订公证结果：

   ```bash
   xcrun stapler staple dist/job-tracker-<version>-macos-<arch>.dmg
   ```

如果没有 Apple Developer 账号，生成的 zip/dmg 可以在本机运行，但分发给别人时 Gatekeeper 会提示未验证。

---

## 5. GitHub Actions 自动发布

仓库里已经包含 `.github/workflows/release.yml`：

- 触发条件：推送形如 `v0.1.0` 的 tag；
- Windows job：构建 release、生成便携 ZIP（可选 Inno Setup 安装包）；
- Linux job：安装图形依赖、构建 release、生成 tar.gz 和 .deb；
- macOS job：构建 release、生成 `JobFlow.app`、zip 和 dmg；
- 最后把所有 `dist/*`（zip / tar.gz / deb / dmg / sha256）上传到对应的 GitHub Release。

发布流程：

```bash
# 1. 修改 Cargo.toml 版本号
# 2. 提交
git add .
git commit -m "release: v0.1.0"
git tag v0.1.0
git push origin main --tags
```

推送 tag 后，GitHub Actions 会自动构建并创建 Release。

---

## 6. 版本号管理

版本号在 `Cargo.toml` 中：

```toml
[package]
version = "0.1.0"
```

建议使用 `cargo-edit` 统一修改：

```bash
cargo install cargo-edit
cargo set-version 0.2.0
```

`Cargo.lock` 会随构建自动更新，记得一起提交。

---

## 7. 发布检查清单

- [ ] 版本号已更新
- [ ] `cargo fmt --check` 通过
- [ ] `cargo test --no-default-features` 通过
- [ ] Windows 便携 ZIP 生成并能在干净机器上运行
- [ ] （可选）Windows 安装程序生成并测试安装/卸载
- [ ] Linux tar.gz / .deb 生成并测试安装
- [ ] Release notes 写清楚平台要求（Vulkan、字体、VC++ 运行库等）
- [ ] 校验和（`sha256sum` / `Get-FileHash`）随 Release 一起提供
