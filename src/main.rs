use std::{collections::HashSet, io::Write, path::PathBuf};

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct Entry {
    name: String,
    host_path: String,
    categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TroveConfig {
    path: String,
    store_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Trove {
    config: TroveConfig,
    entries: HashSet<Entry>,
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
            p.push_str("/");
            p.push_str(&e.name.clone());

            if &get_true_path(&p) == path {
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
        return Err(anyhow!(
            "Could not find a valid .trove file. \r\n Run `trove init <path>` to begin."
        ));
    }

    pub fn create(path: PathBuf) -> Result<Self> {
        // create the trove.conf file
        let mut conf = path.clone();
        conf.push("trove.conf");
        let mut store = path.clone();
        store.push("store");
        let trove = Trove {
            config: TroveConfig {
                path: get_relative_path(&conf),
                store_path: get_relative_path(&store.clone()),
            },
            entries: HashSet::new(),
        };

        let cont = serde_json::to_string_pretty(&trove)?;
        json_to_file(&get_true_path(&trove.config.path), &cont)?;

        if let Err(_) = std::fs::DirBuilder::new().create(store) {}

        trove.create_conf_symlink()?;

        return Ok(trove);
    }

    pub fn save(&self) -> Result<()> {
        let cont = serde_json::to_string_pretty(self)?;
        json_to_file(&get_true_path(&self.config.path), &cont)?;

        return Ok(());
    }

    fn create_conf_symlink(&self) -> Result<()> {
        // create symlink to home dir
        if let Some(mut home) = dirs_next::home_dir() {
            home.push(PathBuf::from(".trove"));
            match symlink::symlink_file(&self.config.path, home) {
                Ok(_) => Ok(()),
                Err(_) => {
                    println!(
                        "Already initialized to: {}",
                        get_true_path(&self.config.path).display()
                    );
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
        let mut host_path_str = host_path.to_string_lossy().to_string();
        if let Some(home) = dirs_next::home_dir() {
            let clean = home.to_string_lossy().to_string();
            if host_path_str.contains(&clean) {
                host_path_str = PathBuf::from(host_path_str.replace(&clean, "$HOME"))
                    .to_string_lossy()
                    .to_string();
            }
        }

        let entry = Entry {
            name: name.into(),
            host_path: host_path_str,
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

    fn add_command(
        &mut self,
        path: &PathBuf,
        name: &String,
        categories: &Option<String>,
    ) -> Result<()> {
        let from_path = get_absolute_path(&path)?;
        let mut to_path = get_true_path(&self.config.store_path);
        to_path.push(name);
        self.add_entry(from_path.clone(), name, categories.clone())?;
        std::fs::rename(&from_path, &to_path)?;

        symlink::symlink_auto(&to_path, &from_path)?;

        return Ok(());
    }

    fn deploy_command(&self, category: &Option<String>, name: &Option<String>) -> Result<()> {
        match (category, name) {
            (None, None) => {
                let mut from_path = String::new();
                for e in &self.entries {
                    from_path.clear();
                    from_path = self.config.store_path.clone();
                    from_path.push_str(&e.name);
                    if let Err(_) = symlink::symlink_auto(&from_path, get_true_path(&e.host_path)) {
                        println!("Could not deploy {}", &e.name);
                    }
                }
            }
            (None, Some(n)) => {
                if let Some(entry) = self.find_entry_by_name(n) {
                    let mut from_path = self.config.store_path.clone();
                    from_path.push_str(&entry.name);
                    if let Err(_) =
                        symlink::symlink_auto(&from_path, get_true_path(&entry.host_path))
                    {
                        println!("Could not deploy {}", &entry.name);
                    }
                } else {
                    return Err(anyhow!("No entry found by that name."));
                }
            }
            (Some(c), None) => {
                let mut from_path = String::new();
                if let Some(entries) = self.find_entry_by_category(c) {
                    for e in entries {
                        from_path.clear();
                        from_path = self.config.store_path.clone();
                        from_path.push_str(&e.name);
                        if let Err(_) =
                            symlink::symlink_auto(&from_path, get_true_path(&e.host_path))
                        {
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

    fn pack_command(&self, category: &Option<String>, name: &Option<String>) -> Result<()> {
        match (category, name) {
            (None, None) => {
                for e in &self.entries {
                    if let Err(_) = symlink::remove_symlink_auto(get_true_path(&e.host_path)) {}
                }
            }
            (None, Some(n)) => {
                if let Some(entry) = self.find_entry_by_name(n) {
                    if let Err(_) = symlink::remove_symlink_auto(get_true_path(&entry.host_path)) {}
                } else {
                    return Err(anyhow!("No entry found by that name."));
                }
            }
            (Some(c), None) => {
                if let Some(entries) = self.find_entry_by_category(c) {
                    for e in entries {
                        if let Err(_) = symlink::remove_symlink_auto(get_true_path(&e.host_path)) {}
                    }
                } else {
                    return Err(anyhow!("No entries found."));
                }
            }
            (Some(_), Some(_)) => return Err(anyhow!("Please specify only one criteria.")),
        }
        return Ok(());
    }

    fn remove_command(&mut self, path: &Option<PathBuf>, name: &Option<String>) -> Result<()> {
        match (path, name) {
            (None, None) => return Err(anyhow!("Need criteria to remove by.")),
            (None, Some(n)) => {
                if let Some(e) = &self.find_entry_by_name(n) {
                    self.remove_entry(e)?;
                    if let Err(_) = symlink::remove_symlink_auto(get_true_path(&e.host_path)) {
                        println!("Symlink does not exists, continuing...",);
                    }
                    let mut from_path = get_true_path(&self.config.store_path);
                    from_path.push(&e.name);
                    std::fs::rename(from_path, get_true_path(&e.host_path))?;
                    return Ok(());
                }
            }
            (Some(p), None) => {
                let abs = get_absolute_path(&p)?;
                if let Some(e) = &self.find_entry_by_path(&abs) {
                    self.remove_entry(e)?;
                    if let Err(_) = symlink::remove_symlink_auto(get_true_path(&e.host_path)) {
                        println!("Symlink does not exists, continuing...",);
                    }
                    let mut from_path = get_true_path(&self.config.store_path);
                    from_path.push(&e.name);
                    std::fs::rename(from_path, get_true_path(&e.host_path))?;
                    return Ok(());
                }
            }
            (Some(_), Some(_)) => return Err(anyhow!("Please specify only one criteria.")),
        }
        return Err(anyhow!("Entry doesn't exists."));
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
            trove.create_conf_symlink()?;
        } else {
            // make a new trove
            let _trove = Trove::create(abs)?;
        }
        return Ok(());
    }
    // get trove
    let mut trove = Trove::load(None)?;
    // run normal command workflows
    match &cli.command {
        Command::Remove { path, name } => trove.remove_command(path, name),
        Command::Deploy { category, name } => trove.deploy_command(category, name),
        Command::Pack { category, name } => trove.pack_command(category, name),
        Command::Status => {
            println!("{:?}", &trove);
            return Ok(());
        }
        Command::Add {
            path,
            name,
            categories,
        } => trove.add_command(path, name, categories),
        _ => unreachable!("Invalid Command"),
    }
}

//util functions
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

pub fn get_absolute_path(rel: &PathBuf) -> Result<PathBuf> {
    // converts from relative path to absolute
    let mut path = std::env::current_dir()?;
    path.push(rel);
    // this also Err if path doesn't exist
    match std::fs::canonicalize(path) {
        Ok(r) => return Ok(r),
        Err(_) => return Err(anyhow!("Path does not exist or isn't a directory.")),
    }
}

pub fn get_true_path(path: &String) -> PathBuf {
    // converts absolute paths with $HOME shorthands to full paths
    if let Some(home) = dirs_next::home_dir() {
        let clean = home.to_string_lossy().to_string();
        if path.contains("$HOME") {
            return PathBuf::from(path.replace("$HOME", &clean));
        }
    }
    return PathBuf::from(path);
}

fn get_relative_path(path: &PathBuf) -> String {
    // converts full paths to relative paths with $HOME shorthands
    let mut path_str = path.to_string_lossy().to_string();
    if let Some(home) = dirs_next::home_dir() {
        let clean = home.to_string_lossy().to_string();
        if path_str.contains(&clean) {
            path_str = PathBuf::from(path_str.replace(&clean, "$HOME"))
                .to_string_lossy()
                .to_string();
        }
    }
    return path_str;
}
