# JobFlow Windows 打包脚本
#
# 用法：
#   .\scripts\package-windows.ps1                     # release 构建 + 便携 ZIP
#   .\scripts\package-windows.ps1 -SkipBuild          # 只重新打包
#   .\scripts\package-windows.ps1 -NoDefaultFeatures  # 跳过 windows-manifest（没有 rc.exe 时）
#   .\scripts\package-windows.ps1 -WindowsGui         # 连 debug 构建也隐藏控制台（release 默认已隐藏）
#   .\scripts\package-windows.ps1 -Installer          # 同时用 Inno Setup 生成安装程序

[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [switch]$NoDefaultFeatures,
    [switch]$WindowsGui,
    [switch]$Installer
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

# 从 Cargo.toml 读取版本号
$versionMatch = Select-String -Path "Cargo.toml" -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
if (-not $versionMatch) {
    throw "无法从 Cargo.toml 读取版本号"
}
$Version = $versionMatch.Matches.Groups[1].Value

function Find-Fxc {
    if ($env:GPUI_FXC_PATH -and (Test-Path $env:GPUI_FXC_PATH)) {
        return $env:GPUI_FXC_PATH
    }
    $cmd = Get-Command "fxc.exe" -ErrorAction SilentlyContinue
    if ($cmd) {
        return $cmd.Source
    }
    $roots = @(
        (Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"),
        (Join-Path $env:ProgramFiles "Windows Kits\10\bin")
    )
    foreach ($root in $roots) {
        if ($root -and (Test-Path $root)) {
            $candidates = Get-ChildItem -Path $root -Filter "fxc.exe" -Recurse -ErrorAction SilentlyContinue
            # 优先 x64 版本
            $found = $candidates |
                Where-Object { $_.FullName -match "\\x64\\" } |
                Sort-Object FullName -Descending | Select-Object -First 1
            if (-not $found) {
                $found = $candidates | Sort-Object FullName -Descending | Select-Object -First 1
            }
            if ($found) {
                return $found.FullName
            }
        }
    }
    return $null
}

$TargetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { "target" }
$Exe = Join-Path $TargetDir "release\job-tracker.exe"
$Dist = Join-Path $Root "dist"
$Stage = Join-Path $Dist ".stage\job-tracker-$Version-windows-x86_64"
$Zip = Join-Path $Dist "job-tracker-$Version-windows-x86_64.zip"

Write-Host "JobFlow 打包" -ForegroundColor Cyan
Write-Host "  版本号: $Version"
Write-Host "  目标平台: windows-x86_64"
Write-Host "  输出目录: $Dist"

if (-not $SkipBuild) {
    $fxc = Find-Fxc
    if (-not $fxc) {
        throw "未找到 fxc.exe（Windows SDK 的 HLSL 编译器）。请安装 Windows SDK，或设置环境变量 GPUI_FXC_PATH 指向 fxc.exe。"
    }
    $env:GPUI_FXC_PATH = $fxc
    Write-Host "  GPUI_FXC_PATH: $fxc"

    $cargoArgs = @("build", "--release")
    if ($NoDefaultFeatures) {
        $cargoArgs += "--no-default-features"
    }
    if ($WindowsGui) {
        # release 构建本来就是 GUI 子系统（不弹控制台），这个开关只对 debug 有意义
        $cargoArgs += @("--features", "windows-gui")
    }
    Write-Host ""
    Write-Host "[1/4] cargo $($cargoArgs -join ' ')" -ForegroundColor Yellow
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build 失败"
    }
}
else {
    Write-Host ""
    Write-Host "[1/4] 跳过构建（-SkipBuild）" -ForegroundColor Yellow
}

if (-not (Test-Path $Exe)) {
    throw "找不到 $Exe，请先执行 cargo build --release"
}

Write-Host ""
Write-Host "[2/4] 准备发布目录" -ForegroundColor Yellow
$StageRoot = Join-Path $Dist ".stage"
if (Test-Path $StageRoot) {
    Remove-Item -Recurse -Force $StageRoot
}
New-Item -ItemType Directory -Force -Path $Stage | Out-Null
Copy-Item $Exe (Join-Path $Stage "job-tracker.exe")
Copy-Item "README.md" $Stage
Copy-Item "LICENSE" $Stage

Write-Host ""
Write-Host "[3/4] 生成 ZIP" -ForegroundColor Yellow
if (Test-Path $Zip) {
    Remove-Item -Force $Zip
}
Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip
Remove-Item -Recurse -Force $StageRoot
Get-FileHash $Zip -Algorithm SHA256 |
    ForEach-Object { "$($_.Hash)  $([System.IO.Path]::GetFileName($Zip))" } |
    Set-Content "$Zip.sha256"

Write-Host ""
Write-Host "[4/4] 完成" -ForegroundColor Green
Write-Host "  压缩包: $Zip"
Write-Host "  校验和: $Zip.sha256"

if ($Installer) {
    Write-Host ""
    Write-Host "生成 Inno Setup 安装程序..." -ForegroundColor Yellow
    $iscc = Get-Command "iscc" -ErrorAction SilentlyContinue
    if (-not $iscc) {
        throw "未找到 iscc.exe，请先安装 Inno Setup 并加入 PATH"
    }
    $iss = Join-Path $Root "packaging\windows\job-tracker.iss"
    & $iscc.Source "/DMyAppVersion=$Version" $iss
    if ($LASTEXITCODE -ne 0) {
        throw "Inno Setup 编译失败"
    }
    Write-Host "安装程序已生成到 dist\ 目录" -ForegroundColor Green
}
