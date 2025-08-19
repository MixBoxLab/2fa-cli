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
curl -fsSL https://raw.githubusercontent.com/MixBoxLab/2fa-cli/main/install.sh | sh
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

## 许可证

本项目基于 MIT 许可证授权。
