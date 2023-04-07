use std::{io::Write, path::PathBuf};

use anyhow::{anyhow, Error, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // command to run
    #[command(subcommand)]
    command: Command,
    // path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    name: String,
    host_path: PathBuf,
    categories: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct TroveConfig {
    path: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Trove {
    config: TroveConfig,
    entries: Vec<Entry>,
}

pub fn json_from_file(path: &PathBuf) -> Result<serde_json::Value> {
    let file = std::fs::File::open(path)?;

    let json = serde_json::from_reader(file).expect("JSON was misformatted.");

    return Ok(json);
}

pub fn json_to_file(path: &PathBuf, contents: &str) -> Result<()> {
    let mut file = std::fs::File::create(path)?;
    file.write_all(contents.as_bytes())?;
    return Ok(());
}

impl Trove {
    pub fn get(p: Option<PathBuf>) -> Result<Self> {
        let mut conf = PathBuf::new();
        match p {
            Some(path) => conf = path,
            None => {
                if let Some(mut path) = dirs_next::home_dir() {
                    path.push(PathBuf::from("/.trove"));
                    conf = path;
                }
            }
        }
        if let Ok(path) = get_absolute_path(&conf) {
            let json = json_from_file(&path)?;
            let trove: Trove = serde_json::from_value(json)?;
            return Ok(trove);
        }
        return Err(anyhow!("Could not find a valid .trove file."));
    }

    pub fn create(mut path: PathBuf) -> Result<Self> {
        // create the trove.conf file
        path.push("trove.conf");
        let trove = Trove {
            config: TroveConfig { path },
            entries: Vec::new(),
        };

        let cont = serde_json::to_string_pretty(&trove)?;
        json_to_file(&trove.config.path, &cont)?;

        trove.create_symlink()?;

        return Ok(trove);
    }

    fn create_symlink(&self) -> Result<()> {
        // create symlink to home dir
        if let Some(mut home) = dirs_next::home_dir() {
            home.push(PathBuf::from(".trove"));
            symlink::symlink_file(&self.config.path, home).expect("Could not create symlink.");
            Ok(())
        } else {
            return Err(anyhow!("Could not find home directory."));
        }
    }

    pub fn update(&mut self) -> Result<()> {
        todo!()
    }
}

#[derive(Subcommand, Debug)]
enum Command {
    Init {
        path: PathBuf,
    },
    Add {
        path: PathBuf,
        name: String,
        #[arg(short, long)]
        categories: Option<String>,
    },
    Remove {
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[arg(short, long)]
        name: Option<PathBuf>,
    },
    Deploy {
        #[arg(short, long)]
        category: Option<String>,
        #[arg(short, long)]
        name: Option<String>,
    },
    Pack {
        #[arg(short, long)]
        category: Option<String>,
        #[arg(short, long)]
        name: Option<String>,
    },
    Status,
}

fn get_absolute_path(rel: &PathBuf) -> Result<PathBuf> {
    let mut path = std::env::current_dir()?;
    path.push(rel);
    // this also Err if path doesn't exist
    match std::fs::canonicalize(path) {
        Ok(r) => return Ok(r),
        Err(_) => return Err(anyhow!("Path does not exist or isn't a directory.")),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Command::Init { path } = &cli.command {
        // have to test for this, as all other commands require a trove set up
        let mut conf = path.clone();
        conf.push("trove.conf");
        // check  if the directory exists
        let _ = get_absolute_path(&path)?;
        if let Ok(targ) = get_absolute_path(&conf) {
            // trove exists, just create symlink
            let trove = Trove::get(Some(targ))?;
            trove.create_symlink()?;
        } else {
            // make a new trove
            let trove = Trove::create(conf)?;
        }
        return Ok(());
    } else {
        // get trove
        let trove = Trove::get(None);
    }
    match &cli.command {
        Command::Remove { path, name } => todo!(),
        Command::Deploy { category, name } => todo!(),
        Command::Pack { category, name } => todo!(),
        Command::Status => todo!(),
        Command::Add {
            path,
            name,
            categories,
        } => todo!(),
        _ => unreachable!("Invalid Command"),
    }
}
