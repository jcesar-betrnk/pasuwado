#![deny(warnings)]

use clap::{Parser, Subcommand};
use clipboard_rs::{Clipboard, ClipboardContext};
use directories::ProjectDirs;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use toml::{Table, Value};

const QUAL: &str = "org";
const ORG: &str = "pasuwado";
const APP: &str = env!("CARGO_PKG_NAME");

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new entry for a user at some domain with the password
    Add {
        /// which domain name
        #[arg(short, long)]
        domain: String,
        /// the username
        #[arg(short, long)]
        user: String,
        /// specify the user password
        #[arg(short, long)]
        password: String,
    },
    /// Get a totp digits for the specified domain
    Get {
        /// which domain name
        #[arg(short, long)]
        domain: Option<String>,

        /// which username
        #[arg(short, long)]
        user: Option<String>,
    },
    /// list all the available domains in the entry
    List,
}

fn config_file() -> anyhow::Result<PathBuf> {
    let proj_dirs = ProjectDirs::from(QUAL, ORG, APP).expect("Could not open config file");
    let config_dir = proj_dirs.config_dir();
    let mut filename = config_dir.to_path_buf();
    filename.set_extension("toml");
    Ok(filename)
}

fn write_to_clipboard(content: &str) -> anyhow::Result<()> {
    let ctx = ClipboardContext::new().expect("Could not get access clipboard");
    ctx.set_text(content.to_string())
        .expect("Could not set the text in the clipboard");
    // ISSUE: it seems it need to be read here in order to make it work
    let _clip = ctx.get_text().expect("Could not read the clipboard text");
    Ok(())
}

fn read_toml_table() -> anyhow::Result<Table> {
    let filename = config_file()?;
    if let Ok(toml_content) = fs::read_to_string(&filename) {
        let toml_value: Result<Value, _> = toml::from_str(&toml_content);
        let Ok(Value::Table(table)) = toml_value else {
            panic!("expecting valid key value toml format");
        };
        Ok(table)
    } else {
        Ok(Table::new())
    }
}

fn ensure_config_dir_exists() -> anyhow::Result<()> {
    let config_file = config_file()?;
    let prefix = config_file
        .parent()
        .expect("must have a parent directory for config file");
    match fs::create_dir_all(prefix) {
        Ok(_) => Ok(()),
        Err(_) => {
            panic!("Unable to create directory: {}", prefix.display());
        }
    }
}

fn save_table_to_toml(table: &Table) -> anyhow::Result<()> {
    let content = toml::to_string(table).unwrap();
    let config_file = config_file()?;
    ensure_config_dir_exists()?;
    let mut file = fs::File::create(config_file)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn add_entry(domain: &str, user: &str, password: &str) -> anyhow::Result<()> {
    let mut table = read_toml_table()?;

    if let Some(existing_domain) = table.get_mut(domain) {
        let Value::Table(existing_domain) = existing_domain else {
            panic!("expecting a table");
        };
        // this also overwrites the previous one
        if let Some(_existing_user) = existing_domain.get(user){
            println!("overwriting entry for {user}");
        }
        existing_domain.insert(user.to_string(), password.into());
    } else {
        let mut user_pwd = Table::new();
        user_pwd.insert(user.to_string(), password.into());
        table.insert(domain.to_string(), user_pwd.into());
    }
    save_table_to_toml(&table)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Commands::Add {
            domain,
            user,
            password,
        } => {
            println!("adding: {domain}, {user}, {password}");
            add_entry(&domain, &user, &password)?;
        }
        Commands::Get { domain, user } => {
            println!("get: {domain:?}, {user:?}");
        }
        Commands::List => {
            let table = read_toml_table()?;
            for (key, _) in table {
                println!("{key}");
            }
        }
    }

    Ok(())
}
