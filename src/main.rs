mod config;
mod commit;
mod rsync;
mod cli;
mod status;
use cli::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let command = Command::parse_options(&args);
    let config = config::Config::parse_config();

    match command {
        Command::Create(options) => {
            let _ = rsync::make_commit(&config, options.name.clone(), options.message, options.verbose);
            println!("Committed: {}", options.name);
        },
        Command::Status => {
            rsync::print_status(&config);
        },
        Command::Restore(options) => {
            if options.commit.is_none() || options.commit == Some("latest".to_string()) {
                let latest = commit::Commit::get_latest(&config);
                if latest.is_none() {
                    eprintln!("No Commits to restore from");
                    std::process::exit(1);
                }
                let latest = latest.unwrap();
                let latest_commit_name = latest.get_folder().file_name().unwrap();
                rsync::restore(&config, latest_commit_name.to_str().unwrap().to_string(), options.path, options.verbose);
            } else {
                rsync::restore(&config, options.commit.unwrap(), options.path, options.verbose);
            }
        }
    }
}
