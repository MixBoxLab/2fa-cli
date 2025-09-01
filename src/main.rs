use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use cli_clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::{
    cursor,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, TOTP};

#[derive(Parser)]
#[command(
    name = "2fa",
    author = "MixBoxLab",
    version = env!("CARGO_PKG_VERSION"),
    about = "A simple, fast, and secure command-line tool for generating TOTP codes",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Watch mode: continuously update codes with live countdown
    #[arg(short, long)]
    watch: bool,
    
    /// Copy specific account code to clipboard
    #[arg(short, long)]
    copy: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new 2FA account
    Add {
        /// The name of the account
        name: String,
        /// The secret key for the account
        secret: String,
    },
    /// List all 2FA accounts
    List,
    /// Remove an existing 2FA account
    Remove {
        /// The name of the account to remove
        name: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct Config {
    accounts: HashMap<String, Account>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Account {
    secret: String,
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().context("Failed to find config directory")?;
    let app_config_dir = config_dir.join("2fa-cli");
    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir)?;
    }
    Ok(app_config_dir.join("secrets.json"))
}

fn load_config(path: &PathBuf) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = fs::read_to_string(path)?;
    let config = serde_json::from_str(&content)?;
    Ok(config)
}

fn save_config(path: &PathBuf, config: &Config) -> Result<()> {
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut ctx = ClipboardContext::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize clipboard: {}", e))?;
    ctx.set_contents(text.to_owned())
        .map_err(|e| anyhow::anyhow!("Failed to copy to clipboard: {}", e))?;
    Ok(())
}

fn get_account_code(account: &Account) -> Result<String> {
    let secret_bytes = base32::decode(
        base32::Alphabet::Rfc4648 { padding: false },
        &account.secret,
    )
    .context("Failed to decode secret. Is it valid Base32?")?;

    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes)?;
    let code = totp.generate_current()?;
    Ok(code)
}

fn display_codes_once(config: &Config) -> Result<()> {
    if config.accounts.is_empty() {
        println!("No accounts found. Use '2fa add <name> <secret>' to add one.");
        return Ok(());
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    println!("\n🔑 2FA Codes:");
    println!("{}{}\n", "-".repeat(60), "-".repeat(10));
    
    let mut index = 1;
    for (name, account) in &config.accounts {
        let code = get_account_code(account)?;
        let remaining = 30 - (now % 30);
        println!("{}. {:<18} {:<10} ⏱️  {:02}s", index, name, code, remaining);
        index += 1;
    }
    
    println!("\n💡 提示: 使用 '2fa --copy <账户名>' 复制特定账户的验证码到剪贴板");
    println!("💡 提示: 使用 '2fa --watch' 进入实时模式，支持键盘快捷键复制");
    
    Ok(())
}

fn display_codes_watch(config: &Config) -> Result<()> {
    if config.accounts.is_empty() {
        println!("No accounts found. Use '2fa add <name> <secret>' to add one.");
        return Ok(());
    }

    // 启用原始模式来捕获 Ctrl+C
    terminal::enable_raw_mode()?;
    
    // 清屏
    execute!(io::stdout(), terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    
    execute!(
        io::stdout(),
        cursor::MoveTo(0, 0),
        Print("🔑 2FA Codes (Press Ctrl+C to exit)\n"),
        cursor::MoveTo(0, 1),
        Print(format!("💡 按数字键 1-{} 复制对应账户的验证码\n", config.accounts.len()))
    )?;
    
    loop {
        // 移动到第三行开始显示内容
        execute!(io::stdout(), cursor::MoveTo(0, 2))?;
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let remaining = 30 - (now % 30);
        
        let mut row = 3u16; // 改为3因为我们添加了一行说明
        let mut index = 1;
        for (name, account) in &config.accounts {
            let code = get_account_code(account)?;
            
            // 根据剩余时间改变颜色
            let color = if remaining <= 5 {
                Color::Red
            } else if remaining <= 10 {
                Color::Yellow
            } else {
                Color::Green
            };
            
            execute!(
                io::stdout(),
                cursor::MoveTo(0, row),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("{}. ", index)),
                SetForegroundColor(Color::White),
                Print(format!("{:<18} ", name)),
                SetForegroundColor(Color::Cyan),
                Print(format!("{:<10} ", code)),
                SetForegroundColor(color),
                Print(format!("⏱️  {:02}s", remaining)),
                ResetColor,
                terminal::Clear(ClearType::UntilNewLine) // 清除行尾剩余内容
            )?;
            row += 1;
            index += 1;
        }
        
        // 只有当账户数量为1时才显示进度条，避免多个账户倒计时不同步的问题
        if config.accounts.len() == 1 {
            let progress = 30 - remaining;
            let bar_length = 30usize;
            let filled = (progress * bar_length as u64 / 30) as usize;
            let empty = bar_length - filled;
            
            // 计算当前应该在哪一行（标题占3行，每个账户占1行）
            let current_row = 3 + config.accounts.len() as u16 + 1;
            
            execute!(
                io::stdout(),
                cursor::MoveTo(0, current_row), // 移动到进度条行的最左侧
                SetForegroundColor(Color::DarkGrey),
                Print("["),
                SetForegroundColor(Color::Green),
                Print("█".repeat(filled)),
                SetForegroundColor(Color::DarkGrey),
                Print("·".repeat(empty)),
                Print("] "),
                SetForegroundColor(Color::White),
                Print(format!(" remaining")),
                ResetColor,
                terminal::Clear(ClearType::UntilNewLine) // 清除行尾
            )?;
        } else {
            // 多个账户时，清除进度条区域
            let current_row = 3 + config.accounts.len() as u16 + 1;
            execute!(
                io::stdout(),
                cursor::MoveTo(0, current_row),
                terminal::Clear(ClearType::UntilNewLine)
            )?;
        }
        
        io::stdout().flush()?;
        
        // 检查是否有按键输入
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key_event) = crossterm::event::read()? {
                if key_event.code == crossterm::event::KeyCode::Char('c') 
                    && key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    break;
                }
                
                // 处理数字键复制功能
                if let crossterm::event::KeyCode::Char(ch) = key_event.code {
                    if ch.is_ascii_digit() {
                        let index = ch.to_digit(10).unwrap() as usize;
                        if index > 0 && index <= config.accounts.len() {
                            let accounts: Vec<_> = config.accounts.iter().collect();
                            if let Some((name, account)) = accounts.get(index - 1) {
                                if let Ok(code) = get_account_code(account) {
                                    if copy_to_clipboard(&code).is_ok() {
                                        // 临时显示复制成功消息
                                        let message_row = 3 + config.accounts.len() as u16 + 2;
                                        execute!(
                                            io::stdout(),
                                            cursor::MoveTo(0, message_row),
                                            SetForegroundColor(Color::Green),
                                            Print(format!("✅ 已复制 {} 的验证码: {}", name, code)),
                                            ResetColor
                                        )?;
                                        io::stdout().flush()?;
                                        // 显示消息1秒后清除
                                        thread::sleep(Duration::from_millis(1000));
                                        execute!(
                                            io::stdout(),
                                            cursor::MoveTo(0, message_row),
                                            terminal::Clear(ClearType::UntilNewLine)
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        thread::sleep(Duration::from_millis(900));
    }
    
    // 恢复终端
    terminal::disable_raw_mode()?;
    execute!(io::stdout(), Print("\n\n"))?;
    println!("👋 Goodbye!");
    
    Ok(())
}


fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_path = get_config_path()?;
    let mut config = load_config(&config_path)?;

    if let Some(command) = cli.command {
        match command {
            Commands::Add { name, secret } => {
                let account = Account { secret };
                config.accounts.insert(name.clone(), account);
                save_config(&config_path, &config)?;
                println!("Account '{}' added.", name);
            }
            Commands::List => {
                if config.accounts.is_empty() {
                    println!("No accounts found.");
                } else {
                    println!("Available accounts:");
                    for name in config.accounts.keys() {
                        println!("- {}", name);
                    }
                }
            }
            Commands::Remove { name } => {
                if config.accounts.remove(&name).is_some() {
                    save_config(&config_path, &config)?;
                    println!("Account '{}' removed.", name);
                } else {
                    println!("Account '{}' not found.", name);
                }
            }
        }
    } else if let Some(account_name) = cli.copy {
        // 复制特定账户的验证码
        if let Some(account) = config.accounts.get(&account_name) {
            let code = get_account_code(account)?;
            copy_to_clipboard(&code)?;
            println!("✅ 已复制 {} 的验证码: {} 到剪贴板", account_name, code);
        } else {
            println!("❌ 未找到账户: {}", account_name);
            println!("\n可用账户:");
            for name in config.accounts.keys() {
                println!("- {}", name);
            }
        }
    } else {
        if cli.watch {
            display_codes_watch(&config)?;
        } else {
            display_codes_once(&config)?;
        }
    }

    Ok(())
}
