[English](README.md) | [简体中文](README-zh_CN.md)

# 2FA CLI

A simple, fast, and secure command-line tool for generating Time-Based One-Time Passwords (TOTP), written in Rust.

## Features

- **Add Accounts**: Easily add new 2FA accounts with a name and a secret key.
- **List Accounts**: View all of your configured accounts.
- **Remove Accounts**: Delete accounts you no longer need.
- **Show Codes**: Display the current 2FA codes for all accounts, along with a countdown timer for expiration.
- **Secure Storage**: Secrets are stored locally in your system's standard configuration directory.

## Installation

### Quick Install (Recommended)

For users without Rust installed, you can install `2fa-cli` with a single command:

```sh
curl -fsSL https://raw.githubusercontent.com/MixBoxLab/2fa-cli/main/install.sh | sh
```

This script will automatically download the appropriate binary for your platform and install it to your local bin directory.

### Install from Source

If you have Rust installed, you can build and install from source:

1.  Clone the repository:
    ```sh
    git clone https://github.com/MixBoxLab/2fa-cli.git
    cd 2fa-cli
    ```

2.  Install using `cargo`:
    ```sh
    cargo install --path .
    ```

This will compile the project and place the `2fa-cli` executable in your Cargo bin directory (`~/.cargo/bin`). Make sure this directory is in your shell's `PATH`.

### Manual Download

You can also download pre-built binaries from the [releases page](https://github.com/MixBoxLab/2fa-cli/releases).

## Usage

### Displaying Codes

To see the current TOTP codes for all your accounts, simply run the command without any subcommands:

```sh
2fa
```

**Example Output:**

```
github               981414     Expires in: 8s
google               123456     Expires in: 21s
```

### Live Watch Mode

For a continuously updating display with real-time countdown and progress bar, use the `--watch` or `-w` flag:

```sh
2fa --watch
```

This will show:
- 🔑 Real-time updating codes with color-coded countdown
- ⏱️ Live countdown timer (green → yellow → red as time expires)
- 📊 Progress bar showing time until next refresh
- 🎯 Press Ctrl+C to exit gracefully

**Watch Mode Features:**
- Codes automatically refresh every 30 seconds
- Color changes based on remaining time (green > 10s, yellow 5-10s, red < 5s)
- Visual progress bar at the bottom
- Clean terminal interface with live updates

### Adding an Account

Use the `add` subcommand to add a new account. The secret key should be the Base32 encoded string provided by your service provider.

```sh
2fa add <ACCOUNT_NAME> <SECRET_KEY>
```

**Example:**

```sh
2fa add github GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ
```

### Listing Accounts

To see a list of all saved account names, use the `list` subcommand:

```sh
2fa list
```

**Example Output:**

```
Available accounts:
- github
- google
```

### Removing an Account

To remove an account, use the `remove` subcommand with the account name:

```sh
2fa remove <ACCOUNT_NAME>
```

**Example:**

```sh
2fa remove github
```

## Building from Source

If you want to build the project without installing it, you can use:

```sh
cargo build --release
```

The executable will be located at `target/release/2fa`.

## License

This project is licensed under the MIT License.
