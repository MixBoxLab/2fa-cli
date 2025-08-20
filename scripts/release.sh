#!/bin/bash

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_message() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# 显示使用方法
show_usage() {
    echo "Usage: $0 [major|minor|patch]"
    echo ""
    echo "Examples:"
    echo "  $0 patch   # 0.1.0 -> 0.1.1"
    echo "  $0 minor   # 0.1.0 -> 0.2.0"
    echo "  $0 major   # 0.1.0 -> 1.0.0"
    echo ""
    echo "If no argument is provided, defaults to 'patch'"
}

# 检查是否有未提交的更改
check_git_status() {
    if [[ -n $(git status --porcelain) ]]; then
        print_message $RED "❌ Error: You have uncommitted changes. Please commit or stash them first."
        git status --short
        exit 1
    fi
}

# 获取当前版本
get_current_version() {
    grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/'
}

# 计算新版本
calculate_new_version() {
    local current_version=$1
    local bump_type=$2
    
    # 分解版本号
    local major=$(echo $current_version | cut -d. -f1)
    local minor=$(echo $current_version | cut -d. -f2)
    local patch=$(echo $current_version | cut -d. -f3)
    
    case $bump_type in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
        *)
            print_message $RED "❌ Error: Invalid bump type '$bump_type'. Use major, minor, or patch."
            exit 1
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# 更新 Cargo.toml 中的版本
update_cargo_version() {
    local new_version=$1
    print_message $BLUE "📝 Updating Cargo.toml version to $new_version..."
    cargo set-version $new_version
}

# 构建和测试
build_and_test() {
    print_message $BLUE "🔨 Building project..."
    cargo build --release
    
    print_message $BLUE "🧪 Running tests..."
    cargo test
    
    print_message $GREEN "✅ Build and tests passed!"
}

# 提交更改
commit_changes() {
    local new_version=$1
    print_message $BLUE "📝 Committing version bump..."
    git add Cargo.toml Cargo.lock
    git commit -m "chore: bump version to v$new_version"
}

# 创建和推送标签
create_and_push_tag() {
    local new_version=$1
    local tag_name="v$new_version"
    
    print_message $BLUE "🏷️  Creating tag $tag_name..."
    git tag -a $tag_name -m "Release $tag_name"
    
    print_message $BLUE "🚀 Pushing changes and tag to remote..."
    git push origin main
    git push origin $tag_name
    
    print_message $GREEN "✅ Tag $tag_name created and pushed successfully!"
}

# 显示发布信息
show_release_info() {
    local old_version=$1
    local new_version=$2
    
    print_message $GREEN "🎉 Release completed successfully!"
    echo ""
    print_message $YELLOW "📊 Release Summary:"
    print_message $YELLOW "  Previous version: $old_version"
    print_message $YELLOW "  New version:      $new_version"
    print_message $YELLOW "  Tag:              v$new_version"
    echo ""
    print_message $BLUE "🔗 GitHub Actions will now build and publish the release automatically."
    print_message $BLUE "   Check the progress at: https://github.com/MixBoxLab/2fa-cli/actions"
    echo ""
    print_message $GREEN "📋 Next steps:"
    print_message $GREEN "   1. Wait for GitHub Actions to complete the build"
    print_message $GREEN "   2. Check the release page: https://github.com/MixBoxLab/2fa-cli/releases"
    print_message $GREEN "   3. Test the installation script: curl -fsSL https://raw.githubusercontent.com/MixBoxLab/2fa-cli/main/install.sh | sh"
}

# 主函数
main() {
    local bump_type=${1:-patch}
    
    # 显示帮助信息
    if [[ "$1" == "-h" || "$1" == "--help" ]]; then
        show_usage
        exit 0
    fi
    
    print_message $GREEN "🚀 Starting release process..."
    
    # 检查 Git 状态
    check_git_status
    
    # 获取当前版本
    local current_version=$(get_current_version)
    print_message $YELLOW "📋 Current version: $current_version"
    
    # 计算新版本
    local new_version=$(calculate_new_version $current_version $bump_type)
    print_message $YELLOW "📋 New version: $new_version"
    
    # 确认操作
    echo ""
    read -p "$(echo -e ${YELLOW}Are you sure you want to release v$new_version? [y/N]: ${NC})" -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_message $YELLOW "❌ Release cancelled."
        exit 0
    fi
    
    # 更新版本
    update_cargo_version $new_version
    
    # 构建和测试
    build_and_test
    
    # 提交更改
    commit_changes $new_version
    
    # 创建和推送标签
    create_and_push_tag $new_version
    
    # 显示发布信息
    show_release_info $current_version $new_version
}

# 运行主函数
main "$@"
