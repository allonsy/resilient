use chrono::offset::Utc;

pub enum Command {
    Create(CreateOptions),
    Status,
}

impl Command {
    pub fn parse_options(args: &[String]) -> Command {
        if args.len() < 2 {
            eprintln!("No command provided");
            std::process::exit(1);
        }

        match args[1].as_str() {
            "create" => {
                let create_options = CreateOptions::parse_options(&args[2..]);
                Command::Create(create_options)
            },
            "status" => {
                Command::Status
            }
            _ => {
                eprintln!("Unknown command: {}", args[1]);
                std::process::exit(1);
            },
        }
    }
}

pub struct CreateOptions {
    pub verbose: bool,
    pub name: String,
    pub message: String,
}

impl CreateOptions {
    fn parse_options(args: &[String]) -> CreateOptions {
        let mut index = 0;
        let arglen = args.len();
        let mut verbose = false;

        let now = Utc::now();
        let formatted_time = now.format("%Y-%m-%d_%H-%M-%S");
        let mut name = format!("{}", formatted_time);

        let mut message = String::new();

        while index < arglen {
            match args[index].as_str() {
                "-v" => {
                    verbose = true;
                },
                "-n" => {
                    if index + 1 >= arglen {
                        eprintln!("No name provided to -n arg");
                        std::process::exit(1);
                    }
                    name = args[index + 1].clone();
                    index += 1;
                },
                "-m" => {
                    if index + 1 >= arglen {
                        eprintln!("No message provided to -m arg");
                        std::process::exit(1);
                    }
                    message = args[index + 1].clone();
                    index += 1;
                }
                _ => {
                    eprintln!("Unknown argument: {}", args[index]);
                    std::process::exit(1);
                }
            }
            index += 1;
        }

        if message.is_empty() {
            eprintln!("Please provide a message via the '-m' argument");
            std::process::exit(1);
        }

        CreateOptions {
            verbose,
            name,
            message
        }
    }
}