use super::config::Config;
use super::commit::Commit;
use super::status;
use std::process::Command;
use std::fs;
use tempdir::TempDir;


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

    let mut rsync_command = Command::new("rsync");
    rsync_command.arg(&flags);
    rsync_command.arg("--delete");
    if !link_arg.is_empty() {
        rsync_command.arg(&link_arg);
    }
    rsync_command.arg(&src_arg);
    rsync_command.arg(&dest_arg);
    let rsync_command = rsync_command.status();
    
    if rsync_command.is_err() {
        eprintln!("Unable to spawn rsync command");
        std::process::exit(1);
    }

    let rsync_output = rsync_command.unwrap();
    if !rsync_output.success() {
        eprintln!("rsync errored out with code: {}", rsync_output.code().unwrap());
        std::process::exit(1);
    }
    
    new_commit.write_commit_file();
    Commit::write_latest(conf, &new_commit);
    new_commit
}

pub fn print_status(conf: &Config) {
    let latest_commit = Commit::get_latest(conf);
    let compare_path = if latest_commit.is_some() {
        latest_commit.unwrap().get_folder().join(DATA_FOLDER_NAME)
    } else {
        let empty_dir = TempDir::new("resilient");
        if empty_dir.is_err() {
            eprintln!("Unable to allocate empty dir!");
            std::process::exit(1);
        }
        empty_dir.unwrap().path().to_path_buf()
    };

    let src_arg = format!("{}/", conf.get_backup_location().display());
    let dest_arg = format!("{}", compare_path.display());

    let flags = "-aAXin";
    let delete_flag = "--delete";

    let rsync_command = Command::new("rsync")
        .arg(flags)
        .arg(delete_flag)
        .arg(&src_arg)
        .arg(&dest_arg)
        .output();
    
    if rsync_command.is_err() {
        eprintln!("Unable to spawn rsync command");
        std::process::exit(1);
    }

    let rsync_output = rsync_command.unwrap();
    if !rsync_output.status.success() {
        eprintln!("rsync command failued with: {}", String::from_utf8(rsync_output.stderr).unwrap());
        std::process::exit(1);
    }

    let output_str = String::from_utf8(rsync_output.stdout);
    if output_str.is_err() {
        eprintln!("Unable to parse rsync output");
        std::process::exit(1);
    }

    let output_str = output_str.unwrap();
    for output_line in output_str.lines() {
        let parsed_change = status::parse_change(output_line);
        if parsed_change.is_some() {
            println!("\t{}", parsed_change.unwrap().get_mod_string());
        }
    }
}

pub fn restore(conf: &Config, commit: String, path: String, verbose: bool) {
    let backups_dir = conf.get_backups_folder();
    let commit_dir = backups_dir.join(&commit);
    if !commit_dir.exists() {
        eprintln!("Cannot find commit: {}", commit);
        std::process::exit(1);
    }


    let src_arg = if path.is_empty() || path == "/".to_string() {
        format!("{}/", commit_dir.join(DATA_FOLDER_NAME).display())
    } else {
        if path.starts_with('/') {
            format!("{}{}", commit_dir.join(DATA_FOLDER_NAME).display(), &path[1..])
        } else {
            format!("{}/{}", commit_dir.join(DATA_FOLDER_NAME).display(), path)
        }
    };

    let backup_location = conf.get_backup_location();
    let dest_arg = if path.is_empty() || path == "/".to_string() {
        format!("{}", backup_location.display())
    } else {
        if path.starts_with('/') {
            format!("{}{}", backup_location.display(), path)
        } else {
            format!("{}", backup_location.join(path).display())
        }
    };

    let flags = if verbose {
        "-aAXv"
    } else {
        "-aAX"
    };

    let rsync_command = Command::new("rsync")
        .arg(flags)
        .arg("--delete")
        .arg(&src_arg)
        .arg(&dest_arg)
        .status();
    
    if rsync_command.is_err() {
        eprintln!("Unable to spawn rsync command");
        std::process::exit(1);
    }

    let rsync_output = rsync_command.unwrap();
    if !rsync_output.success() {
        eprintln!("rsync errored out with code: {}", rsync_output.code().unwrap());
        std::process::exit(1);
    }
}