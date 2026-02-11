mod cli;
mod config;
mod cookies;
mod display;
mod http_client;
mod orly;

use clap::Parser;
use cli::Args;
use cookies::CookieStore;
use display::Display;
use http_client::HttpClient;
use orly::check_login;
use reqwest::Client;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut ui = Display::new(&args.bookid);

    let cookies_path = config::cookies_file();
    if !cookies_path.exists() {
        ui.error_and_exit(
            "cookies.json not found.\n\
             This version requires an existing authenticated session.",
        );
    }

    // Load cookies
    let store = match CookieStore::load_from(&cookies_path) {
        Ok(c) => c,
        Err(e) => ui.error_and_exit(&format!("Failed to read cookies.json: {e}")),
    };

    if store.is_empty() {
        ui.error_and_exit("cookies.json is valid JSON but contains no cookies.");
    }

    let names = store.cookie_names();
    ui.info(&format!(
        "Loaded {} cookies: {}",
        store.len(),
        names.join(", ")
    ));

    // Build the HTTP client with our cookies (no network calls yet).
    let client = match HttpClient::from_store(&store) {
        Ok(c) => c,
        Err(e) => ui.error_and_exit(&format!("Failed to build HTTP client: {e}")),
    };
    ui.info("HTTP client initialized with cookies (no requests performed).");

    // Check whether the cookies work (are we logged in?).
    match check_login(&client).await {
        Ok(true) => ui.info("Login confirmed..."),
        Ok(false) => ui.error_and_exit(
            "Logged out. Cookies could be stale or invalid.\n\
            Try refreshing your cookies.json and trying again.",
        ),
        Err(e) => ui.error_and_exit(&format!("Login check failed: {e}")),
    };

    let output_dir = config::books_root().join(format!("(pending) ({})", args.bookid));

    ui.set_output_dir(output_dir);

    ui.info("Initialization complete.");
    ui.info("No network operations performed in this version.");
}
