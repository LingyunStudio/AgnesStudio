# AgnesStudio 一键打包脚本
# 用法：.\build.ps1
# 产出：dist\AgnesStudio-Setup-x.y.z.exe

$ErrorActionPreference = "Stop"

# 1. 从 Cargo.toml 解析版本号
$cargoToml = Get-Content "Cargo.toml" -Raw
if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
    $version = $matches[1]
} else {
    Write-Error "无法从 Cargo.toml 解析版本号"
    exit 1
}
Write-Host "版本: $version" -ForegroundColor Cyan

# 2. Rust release 构建
Write-Host "正在构建 release…" -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "cargo build --release 失败"
    exit 1
}

# 3. 校验产物
$exePath = "target\release\agnes-studio.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "未找到构建产物: $exePath"
    exit 1
}
Write-Host "构建产物: $exePath ($((Get-Item $exePath).Length) bytes)" -ForegroundColor Green

# 4. 查找 Inno Setup 编译器 ISCC.exe
$iscc = $null
$candidates = @(
    "C:\Program Files (x86)\Inno Setup 6\ISCC.exe",
    "C:\Program Files\Inno Setup 6\ISCC.exe",
    "C:\Program Files (x86)\Inno Setup 5\ISCC.exe"
)
foreach ($c in $candidates) {
    if (Test-Path $c) {
        $iscc = $c
        break
    }
}
if (-not $iscc) {
    # 尝试从 PATH 查找
    $found = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($found) { $iscc = $found.Source }
}
if (-not $iscc) {
    Write-Error @"
未找到 ISCC.exe！
请确认已安装 Inno Setup (https://jrsoftware.org/isinfo.php)
常见路径: C:\Program Files (x86)\Inno Setup 6\ISCC.exe
"@
    exit 1
}
Write-Host "Inno Setup: $iscc" -ForegroundColor Green

# 5. 创建 dist 目录
if (-not (Test-Path "dist")) {
    New-Item -ItemType Directory -Path "dist" | Out-Null
}

# 6. 调用 ISCC 编译安装包
Write-Host "正在编译 Inno Setup 安装包…" -ForegroundColor Yellow
& $iscc /DMyAppVersion="$version" setup.iss
if ($LASTEXITCODE -ne 0) {
    Write-Error "ISCC 编译失败"
    exit 1
}

# 7. 输出结果
$setupExe = "dist\AgnesStudio-Setup-$version.exe"
if (Test-Path $setupExe) {
    $size = (Get-Item $setupExe).Length
    Write-Host "✓ 打包完成: $setupExe ($size bytes)" -ForegroundColor Green
    Write-Host ""
    Write-Host "发布步骤:" -ForegroundColor Cyan
    Write-Host "  1. 在 GitHub 创建 Release，tag 设为 v$version"
    Write-Host "  2. 上传 $setupExe 到该 Release 的 Assets"
    Write-Host "  3. Release title 示例: AgnesStudio v$version"
} else {
    Write-Error "未找到输出文件: $setupExe"
    exit 1
}
