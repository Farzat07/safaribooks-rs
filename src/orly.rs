use crate::http_client::HttpClient;
use anyhow::{bail, Result};
use serde::Deserialize;

pub const PROFILE_URL: &str = "https://learning.oreilly.com/profile/";

/// Minimal subset of the book that we care about.
#[derive(Debug, Deserialize)]
pub struct BookInfo {
    pub title: String,
    pub web_url: String,
}

/// Check whether cookies keep us logged in by fetching the profile page.
/// Returns:
/// - Ok(true)  => HTTP 200 (assume logged in)
/// - Ok(false) => Redirect or 401/403 (assume not logged in)
/// - Err(..)   => Network/other error
pub async fn check_login(client: &HttpClient) -> Result<bool> {
    let res = client.client().get(PROFILE_URL).send().await?;
    let status = res.status();

    if status.is_redirection() {
        Ok(false)
    } else if status == 200 {
        Ok(true)
    } else {
        bail!("Profile request returned unexpected status {}", status)
    }
}

/// Build the v1 API URL for the book.
pub fn book_api_url(bookid: &str) -> String {
    format!("https://learning.oreilly.com/api/v1/book/{bookid}")
}

/// Fetch book metadata from the website.
pub async fn fetch_book_info(client: &HttpClient, bookid: &str) -> Result<BookInfo> {
    let url = book_api_url(bookid);
    let res = client.client().get(url).send().await?;
    let status = res.status();

    if status == 200 {
        let info = res.json::<BookInfo>().await?;
        return Ok(info);
    }
    if status == 404 {
        bail!("Book not found (HTTP 404). Please double-check the book ID provided")
    }
    bail!("Got status: {}", status)
}
