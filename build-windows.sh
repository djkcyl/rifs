#!/bin/bash

echo "开始交叉编译 Windows 版本..."

# 检查是否安装了Windows目标
if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
    echo "安装 Windows 交叉编译目标..."
    rustup target add x86_64-pc-windows-gnu
fi

# 检查是否安装了mingw-w64
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "请先安装 mingw-w64:"
    echo "Ubuntu/Debian: sudo apt install mingw-w64"
    echo "CentOS/RHEL: sudo yum install mingw64-gcc"
    echo "Arch: sudo pacman -S mingw-w64-gcc"
    exit 1
fi

# 设置交叉编译环境变量
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++

# 设置优化编译标志
export RUSTFLAGS="-C target-cpu=native"

# 使用最小体积配置构建Windows版本
echo "使用 release-small 配置构建 Windows 版本..."
cargo build --profile release-small --target x86_64-pc-windows-gnu

if [ $? -ne 0 ]; then
    echo "构建失败！"
    exit 1
fi

# 显示文件大小
echo ""
echo "构建完成！文件大小:"
ls -lh target/x86_64-pc-windows-gnu/release-small/rifs.exe

# 如果安装了 upx，尝试压缩
if command -v upx &> /dev/null; then
    echo ""
    echo "使用 UPX 压缩 Windows 二进制文件..."
    upx --best --lzma target/x86_64-pc-windows-gnu/release-small/rifs.exe
    echo "压缩后文件大小:"
    ls -lh target/x86_64-pc-windows-gnu/release-small/rifs.exe
else
    echo ""
    echo "提示: 安装 UPX 可以进一步减少体积 (apt install upx / brew install upx)"
fi

echo ""
echo "Windows 版本构建完成！"
echo "文件位置: target/x86_64-pc-windows-gnu/release-small/rifs.exe"
echo ""
echo "可以将此文件复制到 Windows 系统上运行" 