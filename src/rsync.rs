use super::config::Config;
use super::commit::Commit;
use std::process::Command;
use std::fs;

const DATA_FOLDER_NAME: &str = "data";

pub fn make_commit(conf: &Config, name: String, message: String, verbose: bool) -> Commit {
    let backups_dir = conf.get_backups_folder();
    let new_backup_folder = backups_dir.join(name);
    
    let mkdir_res = fs::create_dir_all(&new_backup_folder);
    if mkdir_res.is_err() {
        eprintln!("Unable to make new backup folder at location: {}", new_backup_folder.display());
        std::process::exit(1);
    }
    let new_commit = Commit::new(new_backup_folder.clone(), message);

    let data_folder = new_backup_folder.join(DATA_FOLDER_NAME);
    let latest_commit = Commit::get_latest(conf);
    let mut flags = "-aAX".to_string();
    if verbose {
        flags += "v";
    }

    let link_arg = if latest_commit.is_some() {
        let latest_commit = latest_commit.unwrap();
        let latest_folder = latest_commit.get_folder();
        format!("--link-dest={}", latest_folder.canonicalize().unwrap().join(DATA_FOLDER_NAME).display())
    } else {
        String::new()
    };

    let src_arg = format!("{}/", conf.get_backup_location().display());
    let dest_arg = format!("{}", data_folder.display());

    let rsync_command = Command::new("rsync")
        .arg(&flags)
        .arg(&link_arg)
        .arg(&src_arg)
        .arg(&dest_arg)
        .output();
    
    if rsync_command.is_err() {
        eprintln!("Unable to spawn rsync command");
        std::process::exit(1);
    }

    let rsync_output = rsync_command.unwrap();
    if !rsync_output.status.success() {
        eprintln!("rsync errored out with: {}", String::from_utf8(rsync_output.stderr).unwrap());
        std::process::exit(1);
    }
    
    new_commit.write_commit_file();
    Commit::write_latest(conf, &new_commit);
    new_commit
}