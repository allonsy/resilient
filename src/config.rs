use std::path::PathBuf;
use std::path::Path;
use toml::Value;
use toml::value::Table;
use dirs;
use std::env;
use std::fs;

const CONFIG_FILE_VAR_NAME: &str = "RESILIENT_CONFIG_PATH";
const CONFIG_FILE_CONFIG_PATH: &str = "resilient/resilient.conf";

const BACKUP_DIR_KEY: &str = "backup_dir";
const REPO_DIR_KEY: &str = "repo_dir";

const BACKUPS_FOLDER: &str = "backups";

pub struct Config {
    backup_dir: PathBuf,
    repo_dir: PathBuf,
}

impl Config {
    pub fn parse_config() -> Config {
        let config_path = Config::get_config_path();
        let config_contents = fs::read_to_string(&config_path);
        if config_contents.is_err() {
            eprintln!("Unable to read config file at location: {}", config_path.display());
            std::process::exit(1);
        }

        let config_contents = config_contents.unwrap();

        let config_value = toml::from_str(&config_contents);
        if config_value.is_err() {
            eprintln!("Unable to parse toml in config");
            std::process::exit(1);
        }

        let config_value: Value = config_value.unwrap();
        if !config_value.is_table() {
            eprintln!("Toml isn't a table");
            std::process::exit(1);
        }

        let toml_table = config_value.as_table().unwrap();
        let backup_dir = get_toml_string_key(&toml_table, BACKUP_DIR_KEY);
        let repo_dir = get_toml_string_key(&toml_table, REPO_DIR_KEY);

        Config {
            backup_dir: PathBuf::from(backup_dir),
            repo_dir: PathBuf::from(repo_dir),
        }
    }

    pub fn get_backups_folder(&self) -> PathBuf {
        self.repo_dir.join(BACKUPS_FOLDER)
    }
    
    pub fn get_backup_location(&self) -> &Path {
        &self.backup_dir
    }

    fn get_config_path() -> PathBuf {
        let config_path = dirs::config_dir();
        if config_path.is_some() {
            let config_path = config_path.unwrap();
            let config_file_path = config_path.join(CONFIG_FILE_CONFIG_PATH);
            if config_file_path.exists() {
                return config_file_path;
            }
        }

        let config_env_var = env::var(CONFIG_FILE_VAR_NAME);
        if config_env_var.is_ok() {
            let config_env_var = config_env_var.unwrap();
            let config_path = PathBuf::from(config_env_var);
            if config_path.exists() {
                return config_path;
            }
        }

        eprintln!("Cannot find config file");
        std::process::exit(1);
    }
}

fn get_toml_string_key(table: &Table, key: &str) -> String {
    let value = table.get(key);
    if value.is_none() {
        eprintln!("Cannot find key '{}' in toml", key);
        std::process::exit(1);
    }

    let value = value.unwrap();
    if !value.is_str() {
        eprintln!("Toml key '{}' isn't a string", key);
        std::process::exit(1);
    }

    return value.as_str().unwrap().to_string();
}