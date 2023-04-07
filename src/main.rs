use std::path::PathBuf;

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
    repo_path: PathBuf,
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

impl Trove {
    pub fn get() -> Result<Self> {
        let mut conf = PathBuf::new();
        if let Some(mut home) = dirs_next::home_dir() {
            home.push(PathBuf::from("/.trove"));

            if let Ok(path) = get_absolute_path(&home) {
                let file = std::fs::File::open(path)?;
            }
        }
        // TODO find .trove file in $HOME
        let trove = Trove {
            entries: Vec::new(),
            config: TroveConfig {
                repo_path: PathBuf::new(),
                path: PathBuf::new(),
            },
        };
        return Ok(trove);
    }

    pub fn create(path: PathBuf) -> Result<Self> {
        // TODO create the trove.conf file
        // TODO create the .trove file in $HOME
        let trove = Trove::get();
        return Ok(trove.unwrap());
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
        let can = get_absolute_path(path)?;
        return Ok(());
    } else {
        // get trove
        let trove = Trove::get();
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
