#!/bin/bash

echo "🍎 RIFS macOS 原生构建脚本"
echo "========================="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# 检查是否在macOS上运行
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}❌ 此脚本只能在 macOS 系统上运行${NC}"
    echo "对于 Linux 系统，请使用 ./build.sh"
    exit 1
fi

# 创建构建目录
BUILD_DIR="build"
echo -e "${BLUE}📁 创建构建目录: $BUILD_DIR${NC}"
rm -rf $BUILD_DIR
mkdir -p $BUILD_DIR

# 获取版本信息
VERSION=$(grep '^version =' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo -e "${CYAN}📦 版本: $VERSION${NC}"

# 支持的目标平台
TARGETS=(
    "x86_64-apple-darwin:macOS-x64:rifs-macos-x64"
    "aarch64-apple-darwin:macOS-ARM64:rifs-macos-arm64"
)

echo ""
echo -e "${BLUE}🎯 将构建以下平台:${NC}"
for target_info in "${TARGETS[@]}"; do
    IFS=':' read -r target platform filename <<< "$target_info"
    echo "  • $platform ($target)"
done

# 构建统计
SUCCESS_COUNT=0
FAILED_COUNT=0
SKIPPED_COUNT=0

# 构建函数
build_target() {
    local target=$1
    local platform=$2
    local filename=$3
    
    echo ""
    echo -e "${BLUE}🔨 构建 $platform ($target)...${NC}"
    
    # 检查目标是否已安装
    if ! rustup target list --installed | grep -q "$target"; then
        echo -e "${YELLOW}📥 安装 $target 目标...${NC}"
        rustup target add "$target"
    fi
    
    # 设置环境变量
    export RUSTFLAGS="-C target-cpu=native"
    
    # 构建
    local build_log=$(mktemp)
    if cargo build --release --target "$target" > "$build_log" 2>&1; then
        # 复制到build目录
        cp "target/$target/release/rifs" "$BUILD_DIR/$filename"
        
        # 获取文件大小
        local size=$(ls -lh "$BUILD_DIR/$filename" | awk '{print $5}')
        echo -e "${GREEN}✅ $platform 构建成功 ($size)${NC}"
        
        rm -f "$build_log"
        ((SUCCESS_COUNT++))
        return 0
    else
        echo -e "${RED}❌ $platform 构建失败${NC}"
        echo "错误日志:"
        cat "$build_log"
        rm -f "$build_log"
        ((FAILED_COUNT++))
        return 1
    fi
}

# 复制附加文件函数
copy_additional_files() {
    echo ""
    echo -e "${BLUE}📄 复制附加文件...${NC}"
    
    # 复制配置文件
    if [ -f "config.toml" ]; then
        cp config.toml "$BUILD_DIR/"
    elif [ -f "config.example.toml" ]; then
        cp config.example.toml "$BUILD_DIR/config.toml"
    fi
    
    # 复制README
    if [ -f "README.md" ]; then
        cp README.md "$BUILD_DIR/"
    fi
}

# 生成构建信息
generate_build_info() {
    echo -e "${BLUE}📊 生成构建信息...${NC}"
    
    cat > "$BUILD_DIR/build-info.txt" << EOF
RIFS macOS 构建信息
==================

版本: $VERSION
构建时间: $(date)
构建平台: macOS (原生)
构建工具: $(rustc --version)

支持的二进制文件:
- macOS x64: rifs-macos-x64
- macOS ARM64: rifs-macos-arm64

使用说明:
1. 将对应架构的二进制文件重命名为 'rifs'
2. 确保 config.toml 配置文件存在
3. 运行: ./rifs

配置文件说明:
参考 config.toml 进行配置调整
EOF
}

echo ""
echo -e "${GREEN}🚀 开始构建...${NC}"
echo "================"

# 构建所有目标
for target_info in "${TARGETS[@]}"; do
    IFS=':' read -r target platform filename <<< "$target_info"
    build_target "$target" "$platform" "$filename"
done

copy_additional_files
generate_build_info

echo ""
echo -e "${BLUE}📊 构建统计:${NC}"
echo -e "  ✅ 成功: ${GREEN}$SUCCESS_COUNT${NC}"
echo -e "  ❌ 失败: ${RED}$FAILED_COUNT${NC}"
echo -e "  ⏭️  跳过: ${YELLOW}$SKIPPED_COUNT${NC}"

if [ $SUCCESS_COUNT -gt 0 ]; then
    echo ""
    echo -e "${GREEN}🎉 macOS 构建完成！${NC}"
    echo "====================="
    echo -e "${BLUE}📁 构建文件位于: $BUILD_DIR/${NC}"
    echo ""
    ls -la "$BUILD_DIR/"
    
    echo ""
    echo -e "${CYAN}💡 使用提示:${NC}"
    echo "  • 选择对应架构的可执行文件"
    echo "  • 确保配置文件 config.toml 存在" 
    echo "  • Intel Mac 使用: rifs-macos-x64"
    echo "  • Apple Silicon 使用: rifs-macos-arm64"
    
    # 计算总大小
    total_size=$(du -sh "$BUILD_DIR" | cut -f1)
    echo -e "${BLUE}📦 总大小: $total_size${NC}"
    echo ""
    echo -e "${GREEN}💡 提示: 可以直接分发 build 文件夹${NC}"
    echo "🚀 使用 'tar czf rifs-v$VERSION-macos.tar.gz build' 打包分发"
else
    echo ""
    echo -e "${RED}❌ 没有成功构建任何平台${NC}"
    exit 1
fi 