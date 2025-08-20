# 2FA CLI (命令行 2FA 工具)

一个使用 Rust 编写的简单、快速且安全的命令行工具，用于生成基于时间的一次性密码 (TOTP)。

## 功能特性

- **添加账户**: 通过名称和密钥轻松添加新的 2FA 账户。
- **列出账户**: 查看所有已配置的账户。
- **移除账户**: 删除您不再需要的账户。
- **显示验证码**: 显示所有账户当前的 2FA 验证码，并附带一个过期倒计时。
- **安全存储**: 密钥被安全地存储在您系统标准的配置目录中。

## 安装

### 快速安装（推荐）

对于没有安装 Rust 的用户，您可以使用一行命令安装 `2fa-cli`：

```sh
curl -fsSL https://raw.githubusercontent.com/MixBoxLab/2fa-cli/main/scripts/install.sh | sh
```

此脚本将自动下载适合您平台的二进制文件，并将其安装到您的本地 bin 目录。

### 从源码安装

如果您已安装 Rust，您可以从源码构建并安装：

1.  克隆此仓库:
    ```sh
    git clone https://github.com/MixBoxLab/2fa-cli.git
    cd 2fa-cli
    ```

2.  使用 `cargo` 安装:
    ```sh
    cargo install --path .
    ```

此命令将会编译项目，并将 `2fa-cli` 可执行文件放置在您的 Cargo `bin` 目录 (`~/.cargo/bin`) 下。请确保此目录已添加到您 shell 的 `PATH` 环境变量中。

### 手动下载

您也可以从 [发布页面](https://github.com/MixBoxLab/2fa-cli/releases) 下载预构建的二进制文件。

## 使用方法

### 显示验证码

要查看所有账户当前的 TOTP 验证码，只需直接运行命令，不带任何子命令：

```sh
2fa
```

**输出示例:**

```
github               981414     Expires in: 8s
google               123456     Expires in: 21s
```

### 实时监控模式

如需获得带有实时倒计时和进度条的连续更新显示，请使用 `--watch` 或 `-w` 参数：

```sh
2fa --watch
```

此模式将显示：
- 🔑 带有彩色倒计时的实时更新验证码
- ⏱️ 实时倒计时器（绿色 → 黄色 → 红色，随时间过期变化）
- 📊 显示距离下次刷新时间的进度条
- 🎯 按 Ctrl+C 优雅退出

**监控模式特性：**
- 验证码每 30 秒自动刷新
- 根据剩余时间变色（绿色 > 10秒，黄色 5-10秒，红色 < 5秒）
- 底部可视化进度条
- 清洁的终端界面与实时更新

### 添加账户

使用 `add` 子命令来添加一个新账户。密钥应该是您的服务提供商所提供的 Base32 编码的字符串。

```sh
2fa add <账户名> <密钥>
```

**示例:**

```sh
2fa add github GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ
```

### 列出账户

要查看所有已保存账户的名称列表，使用 `list` 子命令：

```sh
2fa list
```

**输出示例:**

```
Available accounts:
- github
- google
```

### 移除账户

要移除一个账户，使用 `remove` 子命令并指定账户名：

```sh
2fa remove <账户名>
```

**示例:**

```sh
2fa remove github
```

## 从源码构建

如果您只想构建项目而不进行安装，可以使用：

```sh
cargo build --release
```

生成的可执行文件将位于 `target/release/2fa`。

## 开发

### 创建发布版本

对于维护者，我们提供了自动化发布脚本：

```sh
# 补丁版本发布 (0.1.0 -> 0.1.1)
./scripts/release.sh patch
# 或者简单地使用
./scripts/quick-release.sh

# 次要版本发布 (0.1.0 -> 0.2.0)
./scripts/release.sh minor

# 主要版本发布 (0.1.0 -> 1.0.0)
./scripts/release.sh major
```

发布脚本将：
1. 检查未提交的更改
2. 更新 `Cargo.toml` 中的版本号
3. 运行测试并构建项目
4. 提交版本更新
5. 创建并推送 git 标签
6. 触发 GitHub Actions 构建并发布版本

## 许可证

本项目基于 MIT 许可证授权。
