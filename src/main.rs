use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
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

fn display_codes_once(config: &Config) -> Result<()> {
    if config.accounts.is_empty() {
        println!("No accounts found. Use '2fa add <name> <secret>' to add one.");
        return Ok(());
    }

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    for (name, account) in &config.accounts {
        let secret_bytes = base32::decode(
            base32::Alphabet::Rfc4648 { padding: false },
            &account.secret,
        )
        .context("Failed to decode secret. Is it valid Base32?")?;

        let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes)?;

        let code = totp.generate_current()?;
        let remaining = 30 - (now % 30);
        println!("{:<20} {:<10} Expires in: {}s", name, code, remaining);
    }
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
    
    println!("🔑 2FA Codes (Press Ctrl+C to exit)\n");
    
    loop {
        // 移动到第三行开始显示内容
        execute!(io::stdout(), cursor::MoveTo(0, 2))?;
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let remaining = 30 - (now % 30);
        
        for (name, account) in &config.accounts {
            let secret_bytes = base32::decode(
                base32::Alphabet::Rfc4648 { padding: false },
                &account.secret,
            )
            .context("Failed to decode secret. Is it valid Base32?")?;

            let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret_bytes)?;
            let code = totp.generate_current()?;
            
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
                SetForegroundColor(Color::White),
                Print(format!("{:<20} ", name)),
                SetForegroundColor(Color::Cyan),
                Print(format!("{:<10} ", code)),
                SetForegroundColor(color),
                Print(format!("⏱️  {}s", remaining)),
                ResetColor,
                Print("\n")
            )?;
        }
        
        // 在底部显示进度条
        let progress = 30 - remaining;
        let bar_length = 40usize;
        let filled = (progress * bar_length as u64 / 30) as usize;
        let empty = bar_length - filled;
        
        execute!(
            io::stdout(),
            Print("\n"),
            SetForegroundColor(Color::DarkGrey),
            Print("Progress: ["),
            SetForegroundColor(Color::Green),
            Print("█".repeat(filled)),
            SetForegroundColor(Color::DarkGrey),
            Print("·".repeat(empty)),
            Print(format!("] {}s", remaining)),
            ResetColor,
            Print("                    ") // 清除行尾
        )?;
        
        io::stdout().flush()?;
        
        // 检查是否有按键输入 (Ctrl+C)
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key_event) = crossterm::event::read()? {
                if key_event.code == crossterm::event::KeyCode::Char('c') 
                    && key_event.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                    break;
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
    } else {
        if cli.watch {
            display_codes_watch(&config)?;
        } else {
            display_codes_once(&config)?;
        }
    }

    Ok(())
}
