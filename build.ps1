# Windows PowerShell 构建脚本
Write-Host "开始优化构建..." -ForegroundColor Green

# 设置环境变量进一步优化
$env:RUSTFLAGS = "-C target-cpu=native -C link-arg=-s"

# 使用最小体积配置构建
Write-Host "使用 release-small 配置构建..." -ForegroundColor Yellow
& cargo build --profile release-small

if ($LASTEXITCODE -ne 0) {
    Write-Host "构建失败！" -ForegroundColor Red
    Read-Host "按任意键继续"
    exit 1
}

# 检查是否安装了 upx
$upxPath = Get-Command upx -ErrorAction SilentlyContinue
if ($upxPath) {
    Write-Host "使用 UPX 压缩二进制文件..." -ForegroundColor Cyan
    & upx --best --lzma target\release-small\rifs.exe
} else {
    Write-Host "提示: 安装 UPX 可以进一步减少体积" -ForegroundColor Yellow
    Write-Host "下载地址: https://github.com/upx/upx/releases" -ForegroundColor Yellow
    Write-Host "或使用 scoop: scoop install upx" -ForegroundColor Yellow
    Write-Host "或使用 chocolatey: choco install upx" -ForegroundColor Yellow
}

# 显示文件大小
Write-Host ""
Write-Host "构建完成！文件大小:" -ForegroundColor Green
$fileInfo = Get-Item target\release-small\rifs.exe -ErrorAction SilentlyContinue
if ($fileInfo) {
    $sizeKB = [math]::Round($fileInfo.Length / 1KB, 2)
    $sizeMB = [math]::Round($fileInfo.Length / 1MB, 2)
    Write-Host "$($fileInfo.Name): $($fileInfo.Length) 字节 ($sizeKB KB / $sizeMB MB)" -ForegroundColor White
} else {
    Write-Host "文件未找到" -ForegroundColor Red
}

Write-Host ""
Write-Host "构建优化说明:" -ForegroundColor Green
Write-Host "- 使用了 LTO (链接时优化)" -ForegroundColor Gray
Write-Host "- 优化级别设为 'z' (最小体积)" -ForegroundColor Gray
Write-Host "- 去除了调试符号" -ForegroundColor Gray
Write-Host "- 禁用了默认 features" -ForegroundColor Gray
Write-Host "- panic 时直接中止，减少 unwinding 代码" -ForegroundColor Gray
Write-Host ""
Write-Host "运行命令: target\release-small\rifs.exe" -ForegroundColor Cyan
Read-Host "按任意键继续" 