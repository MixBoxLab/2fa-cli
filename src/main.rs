use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, TOTP};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
        if config.accounts.is_empty() {
            println!("No accounts found. Use '2fa add <name> <secret>' to add one.");
            return Ok(());
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        for (name, account) in config.accounts {
            let secret_bytes = base32::decode(
                base32::Alphabet::Rfc4648 { padding: false },
                &account.secret,
            )
            .context("Failed to decode secret. Is it valid Base32?")?;

            let totp = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                secret_bytes,
            )?;

            let code = totp.generate_current()?;
            let remaining = 30 - (now % 30);
            println!("{:<20} {:<10} Expires in: {}s", name, code, remaining);
        }
    }

    Ok(())
}
