use colored::*;
use std::{fs::File, path::PathBuf};
use tracing::{error, info};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub struct Display {
    pub log_file: PathBuf,
    pub output_dir: Option<PathBuf>,
}

impl Display {
    pub fn new(book_id: &str) -> Self {
        let log_file = std::env::current_dir()
            .unwrap()
            .join(format!("info_{}.log", book_id));

        let d = Self {
            log_file,
            output_dir: None,
        };

        let file = File::create(&d.log_file).expect("Cannot create log file");

        tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(fmt::layer().with_writer(std::io::stdout))
            .with(fmt::layer().with_writer(file).with_ansi(false))
            .init();

        d.intro();
        info!("** Welcome to SafariBooks (Rust) **");
        d
    }

    pub fn intro(&self) {
        let banner = r#"
 ____         __            _ ____            _
/ ___|  __ _ / _| __ _ _ __(_)  _ \ _   _ ___| |_
\___ \ / _` | |_ / _` | '__| | |_) | | | / __| __|
 ___) | (_| |  _| (_| | |  | |  _ <| |_| \__ \ |_
|____/ \__,_|_|  \__,_|_|  |_|_| \_\\__,_|___/\__|
"#;
        println!("{}", banner.yellow());
        println!("{}", "~".repeat(32));
    }

    pub fn info(&self, msg: &str) {
        println!("{} {}", "[*]".yellow(), msg);
        info!("{msg}");
    }

    pub fn error_and_exit(&self, msg: &str) -> ! {
        eprintln!("{} {}", "[!]".on_red().white(), msg);
        error!("{msg}");
        std::process::exit(1);
    }

    pub fn set_output_dir(&mut self, dir: PathBuf) {
        self.output_dir = Some(dir.clone());
        self.info(&format!("Output directory:\n {}", dir.display()));
    }
}
