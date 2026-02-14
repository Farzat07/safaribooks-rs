mod cli;
mod config;
mod cookies;
mod display;
mod epub;
mod http_client;
mod orly;

use clap::Parser;
use cli::Args;
use cookies::CookieStore;
use display::Display;
use epub::EpubSkeleton;
use http_client::HttpClient;
use orly::{check_login, fetch_book_info};

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

    // Retrieve book info.
    ui.info("Retrieving book info...");
    let bookinfo = match fetch_book_info(&client, &args.bookid).await {
        Ok(info) => info,
        Err(e) => ui.error_and_exit(&format!("Failed to fetch book info: {}", e)),
    };
    ui.info(&format!("{:#?}", bookinfo));

    let skeleton = EpubSkeleton::plan(&config::books_root(), &bookinfo.title, &args.bookid);
    ui.set_output_dir(skeleton.root.clone());

    // Create directories and required files
    if let Err(e) = (|| -> anyhow::Result<()> {
        skeleton.create_dirs()?;
        skeleton.write_mimetype()?;
        skeleton.write_container_xml()?;
        Ok(())
    })() {
        ui.error_and_exit(&format!("EPUB skeleton creation failed: {e}"));
    }
    ui.info("EPUB skeleton ready (mimetype + META-INF/container.xml + OEBPS/).");

    ui.info("Initialization complete.");
    ui.info("No network operations performed in this version.");
}
