mod cli;
mod config;
mod display;

use clap::Parser;
use cli::Args;
use display::Display;

fn main() {
    let args = Args::parse();

    let mut ui = Display::new(&args.bookid);

    let cookies = config::cookies_file();
    if !cookies.exists() {
        ui.error_and_exit(
            "cookies.json not found.\n\
             This version requires an existing authenticated session.",
        );
    }

    ui.info(&format!("Using cookies file: {}", cookies.display()));

    let output_dir = config::books_root().join(format!("(pending) ({})", args.bookid));

    ui.set_output_dir(output_dir);

    ui.info("Initialization complete.");
    ui.info("No network operations performed in this version.");
}
