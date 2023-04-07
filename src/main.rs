use std::{collections::HashSet, io::Write, path::PathBuf};

use anyhow::{anyhow, Error, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    // command to run
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Entry {
    name: String,
    host_path: PathBuf,
    categories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TroveConfig {
    path: PathBuf,
    store_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct Trove {
    config: TroveConfig,
    entries: HashSet<Entry>,
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
    pub fn find_entry_by_name(&self, name: &str) -> Option<Entry> {
        for e in &self.entries {
            if e.name == name {
                return Some(e.clone());
            }
        }
        return None;
    }

    pub fn find_entry_by_path(&self, path: &PathBuf) -> Option<Entry> {
        for e in &self.entries {
            let mut p = self.config.store_path.clone();
            p.push(e.name.clone());

            if &p == path {
                return Some(e.clone());
            }
        }
        return None;
    }

    pub fn find_entry_by_category(&self, category: &String) -> Option<HashSet<Entry>> {
        // might be better to do this functionally
        let mut out = HashSet::new();
        for e in &self.entries {
            if e.categories.contains(category) {
                out.insert(e.to_owned());
            }
        }
        if out.len() == 0 {
            return None;
        } else {
            return Some(out);
        }
    }

    pub fn load(p: Option<PathBuf>) -> Result<Self> {
        let mut conf = PathBuf::new();
        match p {
            Some(path) => conf = path,
            None => {
                if let Some(mut path) = dirs_next::home_dir() {
                    path.push(".trove");
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

    pub fn create(path: PathBuf) -> Result<Self> {
        // create the trove.conf file
        let mut conf = path.clone();
        conf.push("trove.conf");
        let mut store = path.clone();
        store.push("store");
        let trove = Trove {
            config: TroveConfig {
                path: conf,
                store_path: store.clone(),
            },
            entries: HashSet::new(),
        };

        let cont = serde_json::to_string_pretty(&trove)?;
        json_to_file(&trove.config.path, &cont)?;

        std::fs::DirBuilder::new().create(store)?;

        trove.create_symlink()?;

        return Ok(trove);
    }

    pub fn save(&self) -> Result<()> {
        let cont = serde_json::to_string_pretty(self)?;
        json_to_file(&self.config.path, &cont)?;

        return Ok(());
    }

    fn create_symlink(&self) -> Result<()> {
        // create symlink to home dir
        if let Some(mut home) = dirs_next::home_dir() {
            home.push(PathBuf::from(".trove"));
            match symlink::symlink_file(&self.config.path, home) {
                Ok(_) => Ok(()),
                Err(_) => {
                    println!("Already initialized to: {}", &self.config.path.display());
                    Ok(())
                }
            }
        } else {
            return Err(anyhow!("Could not find home directory."));
        }
    }

    fn add_entry(&mut self, path: PathBuf, name: &str, categories: Option<String>) -> Result<()> {
        let cats: Vec<String> = match categories {
            Some(s) => {
                // split on commas
                let split: Vec<String> = s
                    .split(",")
                    .filter(|x| !x.is_empty())
                    .map(|s| s.to_owned())
                    .collect::<Vec<String>>();
                split
            }
            None => Vec::new(),
        };
        // check if the name/path is already loaded
        if let Some(_) = self.find_entry_by_name(name) {
            return Err(anyhow!("Entry by that name already exists."));
        }
        if let Some(_) = self.find_entry_by_path(&path) {
            return Err(anyhow!("Entry with that path already exists."));
        }
        let host_path = get_absolute_path(&path)?;

        let entry = Entry {
            name: name.into(),
            host_path,
            categories: cats,
        };

        self.entries.insert(entry);
        self.save()?;

        return Ok(());
    }

    fn remove_entry(&mut self, entry: &Entry) -> Result<()> {
        self.entries.remove(entry);

        self.save()?;
        Ok(())
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
        name: Option<String>,
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
        // check  if the directory exists
        let abs = get_absolute_path(&path)?;
        let mut conf = abs.clone();
        conf.push("trove.conf");
        if let Ok(targ) = get_absolute_path(&conf) {
            // trove exists, just create symlink
            let trove = Trove::load(Some(targ))?;
            trove.create_symlink()?;
        } else {
            // make a new trove
            let trove = Trove::create(abs)?;
        }
        return Ok(());
    }
    // get trove
    let mut trove = Trove::load(None)?;
    match &cli.command {
        Command::Remove { path, name } => {
            match (path, name) {
                (None, None) => return Err(anyhow!("Need criteria to remove by.")),
                (None, Some(n)) => {
                    if let Some(e) = &trove.find_entry_by_name(n) {
                        trove.remove_entry(e)?;
                        symlink::remove_symlink_auto(&e.host_path)?;
                        let mut from_path = trove.config.store_path.clone();
                        from_path.push(&e.name);
                        std::fs::rename(from_path, &e.host_path)?;
                        return Ok(());
                    }
                }
                (Some(p), None) => {
                    let abs = get_absolute_path(&p)?;
                    if let Some(e) = &trove.find_entry_by_path(&abs) {
                        trove.remove_entry(e)?;
                        symlink::remove_symlink_auto(&e.host_path)?;
                        let mut from_path = trove.config.store_path.clone();
                        from_path.push(&e.name);
                        std::fs::rename(from_path, &e.host_path)?;
                        return Ok(());
                    }
                }
                (Some(_), Some(_)) => return Err(anyhow!("Please specify only one criteria.")),
            }
            return Err(anyhow!("Entry doesn't exists."));
        }
        Command::Deploy { category, name } => {
            match (category, name) {
                (None, None) => {
                    let mut from_path = PathBuf::new();
                    for e in &trove.entries {
                        from_path.clear();
                        from_path = trove.config.store_path.clone();
                        from_path.push(&e.name);
                        if let Ok(_) = symlink::symlink_auto(&from_path, &e.host_path) {
                        } else {
                            println!("Could not deploy {}", &e.name);
                        }
                    }
                }
                (None, Some(n)) => {
                    if let Some(entry) = trove.find_entry_by_name(n) {
                        let mut from_path = trove.config.store_path.clone();
                        from_path.push(&entry.name);
                        if let Ok(_) = symlink::symlink_auto(&from_path, &entry.host_path) {
                        } else {
                            println!("Could not deploy {}", &entry.name);
                        }
                    } else {
                        return Err(anyhow!("No entry found by that name."));
                    }
                }
                (Some(c), None) => {
                    let mut from_path = PathBuf::new();
                    if let Some(entries) = trove.find_entry_by_category(c) {
                        for e in entries {
                            from_path.clear();
                            from_path = trove.config.store_path.clone();
                            from_path.push(&e.name);
                            if let Ok(_) = symlink::symlink_auto(&from_path, &e.host_path) {
                            } else {
                                println!("Could not deploy {}", &e.name);
                            }
                        }
                    } else {
                        return Err(anyhow!("No entries found."));
                    }
                }
                (Some(_), Some(_)) => return Err(anyhow!("Please specify only one criteria.")),
            }

            return Ok(());
        }
        Command::Pack { category, name } => todo!(),
        Command::Status => {
            println!("{:?}", &trove);
            return Ok(());
        }
        Command::Add {
            path,
            name,
            categories,
        } => {
            let from_path = get_absolute_path(&path)?;
            let mut to_path = trove.config.store_path.clone();
            to_path.push(name);
            trove.add_entry(path.clone(), name, categories.clone())?;
            std::fs::rename(&from_path, &to_path)?;

            symlink::symlink_auto(&to_path, &from_path)?;

            return Ok(());
        }
        _ => unreachable!("Invalid Command"),
    }
}
