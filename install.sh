#!/bin/bash

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# 检测操作系统和架构
detect_platform() {
    local os
    local arch
    
    case "$(uname -s)" in
        Darwin*)    os="apple-darwin" ;;
        Linux*)     os="unknown-linux-gnu" ;;
        CYGWIN*|MINGW*|MSYS*) os="pc-windows-msvc" ;;
        *)          
            print_message $RED "Error: Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
    
    case "$(uname -m)" in
        x86_64)     arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *)          
            print_message $RED "Error: Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac
    
    echo "${arch}-${os}"
}

# 获取最新版本号
get_latest_version() {
    curl -s "https://api.github.com/repos/your-username/2fa-cli/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
}

# 主安装函数
install_2fa_cli() {
    print_message $GREEN "🚀 安装 2FA CLI..."
    
    # 检测平台
    local platform=$(detect_platform)
    print_message $YELLOW "📱 检测到平台: $platform"
    
    # 获取最新版本
    local version=$(get_latest_version)
    if [ -z "$version" ]; then
        print_message $RED "❌ 无法获取最新版本信息"
        exit 1
    fi
    print_message $YELLOW "📦 最新版本: $version"
    
    # 构建下载 URL
    local binary_name="2fa-cli"
    if [[ "$platform" == *"windows"* ]]; then
        binary_name="2fa-cli.exe"
    fi
    
    local download_url="https://github.com/your-username/2fa-cli/releases/download/${version}/2fa-cli-${platform}.tar.gz"
    
    # 创建临时目录
    local tmp_dir=$(mktemp -d)
    cd "$tmp_dir"
    
    print_message $YELLOW "⬇️  下载 $download_url"
    
    # 下载二进制文件
    if ! curl -fsSL "$download_url" -o "2fa-cli.tar.gz"; then
        print_message $RED "❌ 下载失败"
        exit 1
    fi
    
    # 解压
    tar -xzf "2fa-cli.tar.gz"
    
    # 确定安装目录
    local install_dir="$HOME/.local/bin"
    if [[ ":$PATH:" != *":$install_dir:"* ]]; then
        # 尝试其他常见的用户 bin 目录
        if [[ ":$PATH:" == *":$HOME/bin:"* ]]; then
            install_dir="$HOME/bin"
        elif [[ ":$PATH:" == *":$HOME/.cargo/bin:"* ]]; then
            install_dir="$HOME/.cargo/bin"
        fi
    fi
    
    # 创建安装目录
    mkdir -p "$install_dir"
    
    # 安装二进制文件
    mv "$binary_name" "$install_dir/2fa-cli"
    chmod +x "$install_dir/2fa-cli"
    
    # 清理临时文件
    cd - > /dev/null
    rm -rf "$tmp_dir"
    
    print_message $GREEN "✅ 2FA CLI 已成功安装到 $install_dir/2fa-cli"
    
    # 检查 PATH
    if [[ ":$PATH:" != *":$install_dir:"* ]]; then
        print_message $YELLOW "⚠️  $install_dir 不在您的 PATH 中"
        print_message $YELLOW "   请将以下行添加到您的 shell 配置文件 (~/.bashrc, ~/.zshrc, 等):"
        print_message $YELLOW "   export PATH=\"$install_dir:\$PATH\""
        print_message $YELLOW "   然后运行: source ~/.bashrc (或对应的配置文件)"
    fi
    
    print_message $GREEN "🎉 安装完成！您现在可以使用 '2fa-cli --help' 命令了"
}

# 运行安装
install_2fa_cli
