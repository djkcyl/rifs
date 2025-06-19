#!/bin/bash

echo "开始优化构建..."

# 设置环境变量进一步优化
export RUSTFLAGS="-C target-cpu=native -C link-arg=-s"

# 使用最小体积配置构建
echo "使用 release-small 配置构建..."
cargo build --profile release-small

# 如果安装了 upx，使用它进一步压缩
if command -v upx &> /dev/null; then
    echo "使用 UPX 压缩二进制文件..."
    upx --best --lzma target/release-small/rifs
else
    echo "提示: 安装 UPX 可以进一步减少体积 (apt install upx / brew install upx)"
fi

# 显示文件大小
echo "构建完成！文件大小:"
ls -lh target/release-small/rifs

echo ""
echo "构建优化说明:"
echo "- 使用了 LTO (链接时优化)"
echo "- 优化级别设为 'z' (最小体积)"
echo "- 去除了调试符号"
echo "- 禁用了默认 features"
echo "- panic 时直接中止，减少 unwinding 代码"
echo ""
echo "运行命令: ./target/release-small/rifs" 