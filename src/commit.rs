use std::path::Path;
use std::path::PathBuf;
use std::fs;
use super::config::Config;
use std::cmp::Ordering;
use chrono::offset::Utc;

const COMMIT_FILE_NAME: &str = "info.commit";
const LATEST_FILE_NAME: &str = "latest.commit";

pub struct Commit {
    timestamp: i64,
    message: String,
    folder: PathBuf
}

impl Commit {
    pub fn new(folder: PathBuf, message: String) -> Commit {
        let date_time = Utc::now();
        Commit {
            timestamp: date_time.timestamp(),
            message,
            folder
        }
    }

    pub fn get_commits(conf: &Config) -> Vec<Commit> {
        let mut commits = Vec::new();

        let repo_dir = conf.get_backups_folder();
        let backups = repo_dir.read_dir();
        if backups.is_err() {
            eprintln!("Unable to read backups from repo");
            std::process::exit(1);
        }

        for folder in backups.unwrap() {
            if folder.is_err() {
                continue;
            }

            let folder_entry = folder.unwrap();
            let folder_path = folder_entry.path();
            if folder_path.is_dir() {
                commits.push(Commit::parse_commit(&folder_path));
            }
        }

        commits.sort_by(sort_commits);
        commits
    }

    pub fn get_latest(conf: &Config) -> Option<Commit> {
        let latest_file = conf.get_backups_folder().join(LATEST_FILE_NAME);
        if !latest_file.exists() {
            return None;
        }

        let latest_contents = fs::read_to_string(&latest_file);
        if latest_contents.is_err() {
            eprintln!("Unable to read latest commit");
            std::process::exit(1);
        }

        Some(Commit::parse_commit(&PathBuf::from(latest_contents.unwrap())))
    }

    pub fn get_folder(&self) -> &Path {
        &self.folder
    }

    pub fn write_commit_file(&self) {
        let contents = format!("{}\n{}", self.timestamp, self.message);

        if !self.folder.exists() {
            eprintln!("Backup folder: '{}' doesn't exist", self.folder.display());
            std::process::exit(1);
        }

        let write_res = fs::write(&self.folder.join(COMMIT_FILE_NAME), &contents);
        if write_res.is_err() {
            eprintln!("Unable to write info.commit file");
            std::process::exit(1);
        }
    }

    pub fn write_latest(conf: &Config, commit: &Commit) {
        let contents = format!("{}", commit.folder.display());
        let latest_file = conf.get_backups_folder().join(LATEST_FILE_NAME);

        let write_res = fs::write(&latest_file, &contents);
        if write_res.is_err() {
            println!("Unable to write latest commit");
            std::process::exit(1);
        }
    }

    fn parse_commit(path: &Path) -> Commit {
        let commit_file = path.join(COMMIT_FILE_NAME);
        if !commit_file.exists() {
            eprintln!("No commit file found for commit: {}", path.display());
            std::process::exit(1);
        }

        let commit_contents = fs::read_to_string(&commit_file);
        if commit_contents.is_err() {
            eprintln!("Unable to read commit file: {}", commit_file.display());
            std::process::exit(1);
        }

        let commit_contents = commit_contents.unwrap();
        let file_lines:Vec<&str> = commit_contents.lines().collect();

        if file_lines.is_empty() {
            eprintln!("No content in commit file: {}", commit_file.display());
            std::process::exit(1);
        }

        let timestamp_str = file_lines[0];
        let timestamp = timestamp_str.parse::<i64>();
        if timestamp.is_err() {
            eprintln!("Unable to parse '{}' as a timestamp", timestamp_str);
            std::process::exit(1);
        }

        let message_strings = file_lines[1..].iter();
        let mut message = String::new();
        for message_line in message_strings {
            message += message_line;
            message += "\n";
        }

        Commit {
            timestamp: timestamp.unwrap(),
            message,
            folder: path.to_path_buf()
        }
    }
}

fn sort_commits(c1: &Commit, c2: &Commit) -> Ordering {
    if c1.timestamp < c2.timestamp {
        Ordering::Less
    } else if c2.timestamp < c1.timestamp {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}