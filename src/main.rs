use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Parser)]
#[command(
    name = "__goto_bin",
    version,
    about = "Resolve explicit directory aliases for the goto shell function."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(hide = true)]
    Resolve {
        alias: String,
    },
    Add {
        alias: String,
        dir: Option<PathBuf>,
    },
    #[command(alias = "rm")]
    Remove {
        alias: String,
    },
    #[command(alias = "ls")]
    List,
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
struct Store {
    aliases: BTreeMap<String, PathBuf>,
}

#[derive(Debug)]
enum AppError {
    EmptyAlias,
    HomeDirUnavailable,
    ConfigDirUnavailable,
    DirectoryDoesNotExist(PathBuf),
    PathIsNotDirectory(PathBuf),
    UnknownAlias(String),
    AliasNotFound(String),
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAlias => write!(f, "goto: alias cannot be empty"),
            Self::HomeDirUnavailable => write!(f, "goto: could not determine home directory"),
            Self::ConfigDirUnavailable => write!(f, "goto: could not determine config directory"),
            Self::DirectoryDoesNotExist(path) => {
                write!(f, "goto: directory does not exist: {}", path.display())
            }
            Self::PathIsNotDirectory(path) => {
                write!(f, "goto: path is not a directory: {}", path.display())
            }
            Self::UnknownAlias(alias) => write!(f, "goto: unknown alias '{alias}'"),
            Self::AliasNotFound(alias) => write!(f, "goto: no alias '{alias}' found"),
            Self::Io { path, source } => {
                write!(f, "goto: I/O error at {}: {source}", path.display())
            }
            Self::Json { path, source } => {
                write!(
                    f,
                    "goto: malformed aliases store {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Json { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let _ = err.print();
            process::exit(1);
        }
    };

    if let Err(err) = run(cli) {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), AppError> {
    match cli.command.unwrap_or(Command::List) {
        Command::Resolve { alias } => {
            validate_alias(&alias)?;
            let store = Store::load()?;
            let path = store
                .aliases
                .get(&alias)
                .ok_or_else(|| AppError::UnknownAlias(alias.clone()))?;
            print!("{}", path.display());
            io::stdout().flush().map_err(|source| AppError::Io {
                path: PathBuf::from("<stdout>"),
                source,
            })?;
        }
        Command::Add { alias, dir } => {
            validate_alias(&alias)?;
            let path = resolve_dir(dir.unwrap_or_else(|| PathBuf::from(".")))?;
            let mut store = Store::load()?;
            store.aliases.insert(alias.clone(), path.clone());
            store.save()?;
            println!("goto: added '{alias}' -> {}", path.display());
        }
        Command::Remove { alias } => {
            validate_alias(&alias)?;
            let mut store = Store::load()?;
            if store.aliases.remove(&alias).is_none() {
                return Err(AppError::AliasNotFound(alias));
            }
            store.save()?;
            println!("goto: removed '{alias}'");
        }
        Command::List => {
            let store = Store::load()?;
            print_aliases(&store);
        }
    }

    Ok(())
}

impl Store {
    fn load() -> Result<Self, AppError> {
        let path = store_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path).map_err(|source| AppError::Io {
            path: path.clone(),
            source,
        })?;

        serde_json::from_str(&contents).map_err(|source| AppError::Json { path, source })
    }

    fn save(&self) -> Result<(), AppError> {
        let path = store_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| AppError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let contents = serde_json::to_string_pretty(self).map_err(|source| AppError::Json {
            path: path.clone(),
            source,
        })?;

        fs::write(&path, format!("{contents}\n")).map_err(|source| AppError::Io { path, source })
    }
}

fn validate_alias(alias: &str) -> Result<(), AppError> {
    if alias.is_empty() {
        return Err(AppError::EmptyAlias);
    }

    Ok(())
}

fn resolve_dir(path: impl AsRef<Path>) -> Result<PathBuf, AppError> {
    let expanded = expand_tilde(path.as_ref())?;
    let absolute = if expanded.is_absolute() {
        expanded
    } else {
        env::current_dir()
            .map_err(|source| AppError::Io {
                path: PathBuf::from("."),
                source,
            })?
            .join(expanded)
    };

    if !absolute.exists() {
        return Err(AppError::DirectoryDoesNotExist(absolute));
    }

    let canonical = absolute.canonicalize().map_err(|source| AppError::Io {
        path: absolute.clone(),
        source,
    })?;

    if !canonical.is_dir() {
        return Err(AppError::PathIsNotDirectory(canonical));
    }

    Ok(canonical)
}

fn expand_tilde(path: &Path) -> Result<PathBuf, AppError> {
    let text = path.to_string_lossy();
    if text == "~" {
        return dirs::home_dir().ok_or(AppError::HomeDirUnavailable);
    }

    if let Some(rest) = text.strip_prefix("~/") {
        let home = dirs::home_dir().ok_or(AppError::HomeDirUnavailable)?;
        return Ok(home.join(rest));
    }

    Ok(path.to_path_buf())
}

fn store_path() -> Result<PathBuf, AppError> {
    if let Some(config_dir) = env::var_os("GOTO_CONFIG_DIR") {
        return Ok(PathBuf::from(config_dir).join("aliases.json"));
    }

    let config_dir = dirs::config_dir().ok_or(AppError::ConfigDirUnavailable)?;
    Ok(config_dir.join("goto").join("aliases.json"))
}

fn print_aliases(store: &Store) {
    let width = store
        .aliases
        .keys()
        .map(String::len)
        .max()
        .unwrap_or_default();

    for (alias, path) in &store.aliases {
        println!("  {alias:<width$} ->  {}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_absolute_path() {
        let result = resolve_dir(env::temp_dir());

        assert!(result.is_ok());
        assert!(result.expect("temp dir should resolve").is_absolute());
    }

    #[test]
    fn resolve_nonexistent_path() {
        let result = resolve_dir("/this/definitely/does/not/exist");

        assert!(matches!(result, Err(AppError::DirectoryDoesNotExist(_))));
    }

    #[test]
    fn store_roundtrip() {
        let mut store = Store::default();
        store.aliases.insert("test".into(), PathBuf::from("/tmp"));

        let serialized = serde_json::to_string(&store).expect("store should serialize");
        let restored: Store = serde_json::from_str(&serialized).expect("store should deserialize");

        assert_eq!(restored.aliases["test"], PathBuf::from("/tmp"));
    }

    #[test]
    fn expands_home_tilde() {
        let home = dirs::home_dir().expect("home dir should be available for tests");

        assert_eq!(expand_tilde(Path::new("~")).expect("tilde expands"), home);
    }
}
