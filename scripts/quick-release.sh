#!/bin/bash

# 快速发布脚本 - 自动执行补丁版本发布

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}🚀 Quick Release - Patch Version${NC}"
echo -e "${YELLOW}This will automatically bump the patch version and create a release.${NC}"
echo

# 运行完整的发布流程
# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
"$SCRIPT_DIR/release.sh" patch
