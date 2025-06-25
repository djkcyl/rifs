#!/bin/bash

echo "🚀 RIFS 多平台一键构建脚本 (Ubuntu)"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# 支持的目标平台（完全移除macOS）
TARGETS=(
    "x86_64-unknown-linux-gnu:Linux-x64:rifs-linux-x64"
    "aarch64-unknown-linux-gnu:Linux-ARM64:rifs-linux-arm64" 
    "x86_64-pc-windows-gnu:Windows-x64:rifs-windows-x64.exe"
    "i686-pc-windows-gnu:Windows-x86:rifs-windows-x86.exe"
)

# 创建构建目录
BUILD_DIR="build"
echo -e "${BLUE}📁 创建构建目录: $BUILD_DIR${NC}"
rm -rf $BUILD_DIR
mkdir -p $BUILD_DIR

# 获取版本信息
VERSION=$(grep '^version =' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo -e "${CYAN}📦 版本: $VERSION${NC}"

# 构建计数器
BUILT_COUNT=0
FAILED_COUNT=0
SKIPPED_COUNT=0

echo ""
echo -e "${BLUE}🎯 支持的目标平台:${NC}"
for target_info in "${TARGETS[@]}"; do
    IFS=':' read -r target platform filename <<< "$target_info"
    echo "  • $platform ($target)"
done

# 检查是否为Ubuntu系统
check_ubuntu() {
    if ! grep -q "Ubuntu" /etc/os-release 2>/dev/null; then
        echo -e "${YELLOW}⚠️  警告: 此脚本专为Ubuntu系统优化${NC}"
        echo "当前系统: $(cat /etc/os-release | grep PRETTY_NAME | cut -d'"' -f2 2>/dev/null || echo '未知')"
        echo ""
    fi
}

# 检查并安装依赖包
install_dependencies() {
    echo -e "${YELLOW}📦 检查Ubuntu依赖包...${NC}"
    
    local packages_needed=()
    local packages_to_install=()
    
    # 定义需要的包
    packages_needed=(
        "mingw-w64:Windows交叉编译"
        "gcc-aarch64-linux-gnu:ARM64交叉编译"
        "upx-ucl:UPX压缩工具"
    )
    
    # 检查每个包
    for package_info in "${packages_needed[@]}"; do
        IFS=':' read -r package desc <<< "$package_info"
        if ! dpkg -l | grep -q "^ii  $package "; then
            echo -e "${YELLOW}  缺少: $package ($desc)${NC}"
            packages_to_install+=("$package")
        else
            echo -e "${GREEN}  已安装: $package${NC}"
        fi
    done
    
    # 如果有缺少的包，询问是否安装
    if [ ${#packages_to_install[@]} -gt 0 ]; then
        echo ""
        echo -e "${YELLOW}发现缺少的依赖包:${NC}"
        for pkg in "${packages_to_install[@]}"; do
            echo -e "  • $pkg"
        done
        echo ""
        read -p "是否自动安装这些依赖包? [y/N]: " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            echo -e "${BLUE}🔧 安装依赖包...${NC}"
            sudo apt update
            sudo apt install -y "${packages_to_install[@]}"
            echo -e "${GREEN}✅ 依赖包安装完成${NC}"
        else
            echo -e "${YELLOW}⚠️  将跳过需要这些依赖的平台构建${NC}"
        fi
    else
        echo -e "${GREEN}✅ 所有依赖包已安装${NC}"
    fi
    echo ""
}

# 安装必要的Rust目标
install_rust_targets() {
    echo -e "${YELLOW}📥 检查并安装Rust交叉编译目标...${NC}"
    for target_info in "${TARGETS[@]}"; do
        IFS=':' read -r target platform filename <<< "$target_info"
        if ! rustup target list --installed | grep -q "$target"; then
            echo -e "${YELLOW}  安装目标: $target${NC}"
            rustup target add $target
        fi
    done
    echo ""
}

# 检查工具状态的辅助函数
check_tool_status() {
    if command -v "$1" &> /dev/null; then
        echo -e "${GREEN}✅ 已安装${NC}"
    else
        echo -e "${RED}❌ 未安装${NC}"
    fi
}

# 检查Docker Buildx状态
check_buildx_status() {
    if command -v docker &> /dev/null && docker buildx version &> /dev/null; then
        echo -e "${GREEN}✅ 已安装${NC}"
    else
        echo -e "${RED}❌ 未安装${NC}"
    fi
}

# 检查Docker登录状态
check_docker_login() {
    echo -e "${YELLOW}🔐 检查Docker登录状态...${NC}"
    
    # 检查是否已登录
    if ! docker info | grep -q "Username:" 2>/dev/null; then
        echo -e "${RED}❌ 未登录Docker仓库${NC}"
        echo -e "${CYAN}💡 请先运行: docker login${NC}"
        return 1
    fi
    
    # 获取当前登录用户
    local docker_user=$(docker info 2>/dev/null | grep "Username:" | awk '{print $2}' || echo "unknown")
    echo -e "${GREEN}✅ 已登录用户: $docker_user${NC}"
    
    return 0
}

# 检查交叉编译工具
check_cross_tools() {
    echo ""
    echo -e "${BLUE}🔧 交叉编译工具状态:${NC}"
    echo -e "  • mingw-w64 (Windows): $(check_tool_status x86_64-w64-mingw32-gcc)"
    echo -e "  • gcc-aarch64 (ARM64): $(check_tool_status aarch64-linux-gnu-gcc)"
    echo -e "  • Docker (多架构构建): $(check_tool_status docker)"
    echo -e "  • Docker Buildx: $(check_buildx_status)"
    echo ""
}

# 检查Docker是否可用
check_docker() {
    if ! command -v docker &> /dev/null; then
        echo -e "${YELLOW}⚠️  Docker未安装，将跳过Docker构建${NC}"
        return 1
    fi
    
    if ! docker info &> /dev/null; then
        echo -e "${YELLOW}⚠️  Docker守护进程未运行，将跳过Docker构建${NC}"
        return 1
    fi
    
    return 0
}

# Docker多架构构建函数
build_docker_images() {
    local push_enabled=${1:-false}
    
    echo ""
    echo -e "${BLUE}🐳 开始Docker多架构构建...${NC}"
    echo "========================="
    
    # 检查是否有Linux二进制文件
    local has_linux_x64=false
    local has_linux_arm64=false
    
    if [ -f "$BUILD_DIR/rifs-linux-x64" ]; then
        has_linux_x64=true
    fi
    
    if [ -f "$BUILD_DIR/rifs-linux-arm64" ]; then
        has_linux_arm64=true
    fi
    
    if [ "$has_linux_x64" = false ] && [ "$has_linux_arm64" = false ]; then
        echo -e "${YELLOW}⚠️  没有找到Linux二进制文件，跳过Docker构建${NC}"
        return 1
    fi
    
    # 获取Docker用户名并设置镜像标签
    local docker_user=$(docker info 2>/dev/null | grep "Username:" | awk '{print $2}' || echo "")
    local image_name=""
    
    if [ -n "$docker_user" ] && [ "$docker_user" != "unknown" ]; then
        image_name="$docker_user/rifs"
    else
        echo -e "${RED}❌ 无法获取Docker用户名${NC}"
        return 1
    fi
    
    local image_tag="latest"
    local full_tag="$image_name:$image_tag"
    local version_tag="$image_name:v$VERSION"
    
    echo -e "${CYAN}📦 镜像标签: $full_tag, $version_tag${NC}"
    
    # 检查并设置buildx
    echo -e "${YELLOW}🔧 设置Docker buildx...${NC}"
    if ! docker buildx version &> /dev/null; then
        echo -e "${RED}❌ Docker buildx不可用，无法进行多架构构建${NC}"
        return 1
    fi
    
    # 创建或使用multiarch builder
    local builder_name="rifs-multiarch"
    if ! docker buildx ls | grep -q "$builder_name"; then
        echo -e "${YELLOW}  创建multiarch builder...${NC}"
        docker buildx create --name "$builder_name" --driver docker-container --use
    else
        echo -e "${GREEN}  使用已存在的multiarch builder${NC}"
        docker buildx use "$builder_name"
    fi
    
    # 启动builder
    docker buildx inspect --bootstrap
    
    # 准备构建平台列表
    local platforms=()
    if [ "$has_linux_x64" = true ]; then
        platforms+=("linux/amd64")
    fi
    if [ "$has_linux_arm64" = true ]; then
        platforms+=("linux/arm64")
    fi
    
    local platform_list=$(IFS=,; echo "${platforms[*]}")
    echo -e "${CYAN}🎯 构建平台: $platform_list${NC}"
    
    # 根据预设配置确定推送方式
    local push_flag=""
    if [ "$push_enabled" = true ]; then
        push_flag="--push"
        echo -e "${GREEN}✅ 将推送镜像到仓库${NC}"
    else
        push_flag="--load"
        echo -e "${YELLOW}📱 只构建到本地${NC}"
    fi
    
    # 执行Docker构建
    echo ""
    echo -e "${BLUE}🔨 执行Docker多架构构建...${NC}"
    local build_log=$(mktemp)
    
    if docker buildx build \
        --platform "$platform_list" \
        --tag "$full_tag" \
        --tag "$version_tag" \
        $push_flag \
        . > "$build_log" 2>&1; then
        
        echo -e "${GREEN}✅ Docker多架构构建成功${NC}"
        rm -f "$build_log"
        
        # 显示镜像信息
        echo ""
        echo -e "${CYAN}📊 构建的Docker镜像:${NC}"
        echo -e "  • $full_tag"
        echo -e "  • $version_tag"
        echo -e "  • 平台: $platform_list"
        
        if [[ $push_flag == "--push" ]]; then
            echo -e "${GREEN}🚀 镜像已推送到Docker仓库${NC}"
        else
            echo -e "${YELLOW}📱 镜像已构建到本地${NC}"
            echo ""
            echo -e "${CYAN}本地镜像列表:${NC}"
            docker images | grep "$image_name" | head -5
        fi
        
        return 0
    else
        echo -e "${RED}❌ Docker构建失败${NC}"
        echo -e "${RED}错误日志:${NC}"
        cat "$build_log"
        rm -f "$build_log"
        return 1
    fi
}

# 构建函数
build_target() {
    local target=$1
    local platform=$2
    local filename=$3
    
    echo -e "${BLUE}🔨 构建 $platform ($target)...${NC}"
    
    # 设置环境变量
    export RUSTFLAGS="-C target-cpu=native"
    
    # 特殊配置和依赖检查
    case $target in
        *windows*)
            if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
                echo -e "${YELLOW}⚠️  跳过 $platform: 缺少 mingw-w64${NC}"
                ((SKIPPED_COUNT++))
                return 1
            fi
            export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
            export CARGO_TARGET_I686_PC_WINDOWS_GNU_LINKER=i686-w64-mingw32-gcc
            ;;
        *aarch64*linux*)
            if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
                echo -e "${YELLOW}⚠️  跳过 $platform: 缺少 gcc-aarch64-linux-gnu${NC}"
                ((SKIPPED_COUNT++))
                return 1
            fi
            export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
            ;;
    esac
    
    # 执行构建（静默输出，将stdout和stderr重定向到临时文件）
    local build_log=$(mktemp)
    if cargo build --profile release-small --target $target > "$build_log" 2>&1; then
        # 构建成功，清理日志文件
        rm -f "$build_log"
        
        # 确定源文件路径
        local src_file="target/$target/release-small/rifs"
        if [[ $filename == *.exe ]]; then
            src_file="target/$target/release-small/rifs.exe"
        fi
        
        # 复制到构建目录
        cp "$src_file" "$BUILD_DIR/$filename"
        
        # UPX压缩 (仅对支持的格式，静默输出)
        if command -v upx &> /dev/null; then
            case $target in
                *linux*|*windows*)
                    echo -e "${CYAN}🗜️  UPX 压缩 $filename...${NC}"
                    if upx --best --lzma --force "$BUILD_DIR/$filename" &> /dev/null; then
                        echo -e "${GREEN}    压缩完成${NC}"
                    else
                        echo -e "${YELLOW}    压缩失败，保留原文件${NC}"
                    fi
                    ;;
                *)
                    echo -e "${YELLOW}    跳过UPX压缩 (不支持的格式)${NC}"
                    ;;
            esac
        fi
        
        echo -e "${GREEN}✅ $platform 构建成功${NC}"
        ((BUILT_COUNT++))
        return 0
    else
        # 构建失败，显示错误日志
        echo -e "${RED}❌ $platform 构建失败${NC}"
        echo -e "${RED}错误日志:${NC}"
        cat "$build_log"
        rm -f "$build_log"
        ((FAILED_COUNT++))
        return 1
    fi
}

# 主流程
main() {
    # 检查系统
    check_ubuntu
    
    # 安装依赖
    install_dependencies
    
    # 安装Rust目标
    install_rust_targets
    
    # 检查工具
    check_cross_tools
    
    # Docker构建配置
    local docker_build_enabled=false
    local docker_push_enabled=false
    
    echo ""
    echo -e "${BLUE}🐳 Docker多架构构建配置${NC}"
    echo "========================="
    
    # 检查Docker可用性
    if check_docker; then
        read -p "是否启用Docker多架构构建? [y/N]: " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            docker_build_enabled=true
            echo ""
            read -p "是否推送镜像到Docker仓库? [y/N]: " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                # 检查登录状态
                if check_docker_login; then
                    docker_push_enabled=true
                    echo -e "${GREEN}✅ 将推送镜像到仓库${NC}"
                else
                    echo ""
                    read -p "未登录Docker，是否继续只构建到本地? [y/N]: " -n 1 -r
                    echo
                    if [[ $REPLY =~ ^[Yy]$ ]]; then
                        echo -e "${YELLOW}📱 将只构建到本地${NC}"
                    else
                        docker_build_enabled=false
                        echo -e "${RED}❌ 用户取消Docker构建${NC}"
                    fi
                fi
            else
                echo -e "${YELLOW}📱 将只构建到本地${NC}"
            fi
        fi
    else
        echo -e "${YELLOW}⚠️  Docker不可用，跳过Docker构建${NC}"
    fi
    
    # 开始构建所有目标
    echo ""
    echo -e "${GREEN}🚀 开始多平台构建...${NC}"
    echo "==================="
    
    for target_info in "${TARGETS[@]}"; do
        IFS=':' read -r target platform filename <<< "$target_info"
        echo ""
        build_target "$target" "$platform" "$filename"
    done
    
    # 复制其他文件
    echo ""
    echo -e "${CYAN}📄 复制附加文件...${NC}"
    cp README.md $BUILD_DIR/
    
    # Docker多架构构建
    if [ "$docker_build_enabled" = true ]; then
        build_docker_images "$docker_push_enabled"
    fi
    
    # 生成构建信息
    echo -e "${CYAN}📊 生成构建信息...${NC}"
    cat > $BUILD_DIR/build-info.txt << EOF
RIFS 多平台构建信息
==================

版本: $VERSION
构建时间: $(date)
构建主机: $(uname -a)
Rust版本: $(rustc --version)

构建统计:
- 成功: $BUILT_COUNT 个平台
- 失败: $FAILED_COUNT 个平台  
- 跳过: $SKIPPED_COUNT 个平台

支持的平台:
EOF

    # 添加构建的文件信息
    for target_info in "${TARGETS[@]}"; do
        IFS=':' read -r target platform filename <<< "$target_info"
        if [ -f "$BUILD_DIR/$filename" ]; then
            size=$(ls -lh "$BUILD_DIR/$filename" | awk '{print $5}')
            echo "✅ $platform: $filename ($size)" >> $BUILD_DIR/build-info.txt
        else
            echo "❌ $platform: 构建失败或跳过" >> $BUILD_DIR/build-info.txt
        fi
    done
    
    cat >> $BUILD_DIR/build-info.txt << EOF

使用方法:
1. 根据你的操作系统和架构选择对应的可执行文件
2. 修改 config.toml 配置文件
3. 运行程序

平台对应关系:
- Linux x64: ./rifs-linux-x64
- Linux ARM64: ./rifs-linux-arm64
- Windows x64: rifs-windows-x64.exe
- Windows x86: rifs-windows-x86.exe

依赖说明:
- Windows交叉编译需要: mingw-w64
- ARM64交叉编译需要: gcc-aarch64-linux-gnu
- Docker多架构构建需要: docker, docker-buildx

Docker镜像:
EOF

    # 如果启用了Docker，添加Docker镜像信息
    if check_docker &>/dev/null; then
        local docker_user=$(docker info 2>/dev/null | grep "Username:" | awk '{print $2}' || echo "")
        if [ -n "$docker_user" ] && [ "$docker_user" != "unknown" ]; then
            cat >> $BUILD_DIR/build-info.txt << EOF
如果启用了Docker构建，可以使用以下镜像:
- $docker_user/rifs:latest (最新版本)
- $docker_user/rifs:v$VERSION (当前版本)
- 支持平台: linux/amd64, linux/arm64

使用Docker运行:
docker run --rm --pull always -d \
  -p 3000:3000 \
  -v ./uploads:/app/uploads \
  -v ./cache:/app/cache \
  -v ./data:/app/data \
  -v ./config.toml:/app/config.toml \
  $docker_user/rifs:latest
EOF
        else
            cat >> $BUILD_DIR/build-info.txt << EOF
Docker镜像构建需要先登录Docker Hub:
docker login
然后重新运行构建脚本进行Docker构建
EOF
        fi
    else
        cat >> $BUILD_DIR/build-info.txt << EOF
Docker镜像构建需要安装Docker和Docker Buildx
EOF
    fi

    cat >> $BUILD_DIR/build-info.txt << EOF
EOF

    # 显示构建结果
    echo ""
    echo -e "${GREEN}🎉 多平台构建完成！${NC}"
    echo "====================="
    echo ""
    echo -e "${CYAN}📊 构建统计:${NC}"
    echo -e "  • ${GREEN}成功: $BUILT_COUNT 个平台${NC}"
    echo -e "  • ${RED}失败: $FAILED_COUNT 个平台${NC}"
    echo -e "  • ${YELLOW}跳过: $SKIPPED_COUNT 个平台${NC}"
    
    # 检查Docker构建结果
    if [ "$docker_build_enabled" = true ]; then
        echo -e "  • ${BLUE}Docker: 已构建多架构镜像${NC}"
    fi
    echo ""
    echo -e "${CYAN}📁 构建文件位置: $BUILD_DIR/${NC}"
    echo ""
    echo "文件列表:"
    ls -lh $BUILD_DIR/
    
    # 计算总大小
    total_size=$(du -sh $BUILD_DIR | cut -f1)
    echo ""
    echo -e "${GREEN}📦 总大小: $total_size${NC}"
    echo ""
    echo -e "${YELLOW}💡 提示: 可以直接分发 $BUILD_DIR 文件夹${NC}"
    echo -e "${PURPLE}🚀 使用 'tar czf rifs-v$VERSION-multi.tar.gz $BUILD_DIR' 打包分发${NC}"
}

# 运行主流程
main