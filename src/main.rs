#![deny(warnings)]

use clap::{Parser, Subcommand};
use clipboard_rs::{Clipboard, ClipboardContext};
use directories::ProjectDirs;
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use thiserror::Error;
use toml::{Table, Value};
use std::env;
#[macro_use] extern crate log;

const QUAL: &str = "org";
const ORG: &str = "pasuwado";
const APP: &str = env!("CARGO_PKG_NAME");

#[derive(Error, Debug)]
enum Error {
    #[error("You need to specify either a domain or a user")]
    NoneSpecified,
    #[error("There are multiple entries for the domain {domain:?}, you need to specify which user:\n{}", user_list.join("\n"))]
    MultipleMatchingEntry {
        domain: String,
        user_list: Vec<String>,
    },
    #[error("No entry for user: {user:?} under domain: {domain:?}")]
    NoMatch { user: String, domain: String },
    #[error("No entry for domain: {domain:?}")]
    NoMatchingDomain{domain: String},
    #[error("No entry for user: {user:?}")]
    NoMatchingUser{user: String},
    #[error("{0}")]
    IoError(#[from] io::Error),
    #[error("Entry for {user:?} from domain: {domain:?}, already exist, use --force to overwrite it")]
    EntryExist{domain: String, user: String},
}

#[derive(Debug)]
struct Entry {
    #[allow(unused)]
    domain: String,
    #[allow(unused)]
    user: String,
    pwd: String,
}

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
        /// force overwrite if there is already an existing entry
        #[arg(short, long, default_value_t = false)]
        force: bool
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

fn config_file() -> io::Result<PathBuf> {
    let proj_dirs = ProjectDirs::from(QUAL, ORG, APP).expect("Could not open config file");
    let config_dir = proj_dirs.config_dir();
    let mut filename = config_dir.to_path_buf();
    filename.set_extension("toml");
    Ok(filename)
}

fn write_to_clipboard(content: &str) -> Result<(), Error> {
    let ctx = ClipboardContext::new().expect("Could not get access clipboard");
    ctx.set_text(content.to_string())
        .expect("Could not set the text in the clipboard");
    // ISSUE: it seems it need to be read here in order to make it work
    let _clip = ctx.get_text().expect("Could not read the clipboard text");
    Ok(())
}

fn read_toml_table() -> Result<Table, Error> {
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

fn ensure_config_dir_exists() -> Result<(), Error> {
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

fn save_table_to_toml(table: &Table) -> Result<(), Error> {
    let content = toml::to_string(table).unwrap();
    let config_file = config_file()?;
    ensure_config_dir_exists()?;
    let mut file = fs::File::create(config_file)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn add_entry(domain: &str, user: &str, password: &str, force: bool) -> Result<(), Error> {
    let mut table = read_toml_table()?;

    if let Some(existing_domain) = table.get_mut(domain) {
        let Value::Table(existing_domain) = existing_domain else {
            panic!("expecting a table");
        };
        // this also overwrites the previous one
        if let Some(_existing_user) = existing_domain.get(user) {
            if force{
                warn!("Overwriting entry for {user:?} in domain: {domain:?}");
                existing_domain.insert(user.to_string(), password.into());
            }else{
                return Err(Error::EntryExist{domain: domain.to_string(), user: user.to_string()})
            }
        }else{
            existing_domain.insert(user.to_string(), password.into());
        }
    } else {
        let mut user_pwd = Table::new();
        user_pwd.insert(user.to_string(), password.into());
        table.insert(domain.to_string(), user_pwd.into());
    }
    save_table_to_toml(&table)?;
    Ok(())
}

fn find_entry(domain: &Option<String>, user: &Option<String>) -> Result<Entry, Error> {
    let table = read_toml_table()?;
    if let Some(domain) = domain {
        let user_list = table.get(domain);
        match user_list {
            Some(user_list) => {
                let Value::Table(user_list) = user_list else {
                    panic!("must be a string");
                };
                if let Some(user) = user {
                    if let Some(pwd) = user_list.get(user) {
                        let Value::String(pwd) = pwd else {
                            panic!("pwd should be a string!");
                        };
                        Ok(Entry {
                            domain: domain.to_string(),
                            user: user.to_string(),
                            pwd: pwd.to_string(),
                        })
                    } else {
                        Err(Error::NoMatch {
                            user: user.to_string(),
                            domain: domain.to_string(),
                        })
                    }
                } else {
                    if user_list.len() == 1 {
                        let (user, pwd) = user_list.iter().next().unwrap();
                        let Value::String(pwd) = pwd else {
                            panic!("pwd should be a string!");
                        };
                        Ok(Entry {
                            domain: domain.to_string(),
                            user: user.to_string(),
                            pwd: pwd.to_string(),
                        })
                    } else {
                        Err(Error::MultipleMatchingEntry {
                            domain: domain.to_string(),
                            user_list: user_list.iter().map(|(user, _)| user.to_string()).collect(),
                        })
                    }
                }
            }
            None => {
                Err(Error::NoMatchingDomain{domain: domain.to_string()})
            }
        }
    } else {
        // read all the table and see if the user match
        if let Some(user) = user {
            // use the user that matches the first entry in the first domain encountered
            for (domain, user_list) in table {
                if let Some(pwd) = user_list.get(user) {
                    let Value::String(pwd) = pwd else {
                        panic!("password must be string");
                    };
                    return Ok(Entry {
                        domain: domain.to_string(),
                        user: user.to_string(),
                        pwd: pwd.to_string(),
                    });
                }
            }
            Err(Error::NoMatchingUser{user: user.to_string()})
        } else {
            println!("You need to specify either a domain or a username");
            Err(Error::NoneSpecified)
        }
    }
}

fn copy_password_to_clipboard(
    domain: &Option<String>,
    user: &Option<String>,
) -> anyhow::Result<()> {
    match find_entry(domain, user){
        Ok(entry) => {
            println!("found entry: {entry:?}");
            write_to_clipboard(&entry.pwd)?;
        }
        Err(e) => {
            error!("Error: {e}");
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "warn")
    }
    pretty_env_logger::init();
    let args = Args::parse();
    match args.command {
        Commands::Add {
            domain,
            user,
            password,
            force,
        } => {
            info!("adding: {domain}, {user}");
            add_entry(&domain, &user, &password, force)?;
        }
        Commands::Get { domain, user } => {
            info!("get: {domain:?}, {user:?}");
            copy_password_to_clipboard(&domain, &user)?;
        }
        Commands::List => {
            let table = read_toml_table()?;
            for (domain, user_list) in table {
                let Value::Table(user_list) = user_list else {panic!("expecting a table");};
                println!("domain: {domain:?}\n");
                for (user, _pwd) in user_list{
                    println!(" - {user}");
                }
                println!("");
            }
        }
    }

    Ok(())
}
