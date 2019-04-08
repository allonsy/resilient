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
        }
    }
}
