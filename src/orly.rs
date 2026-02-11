use crate::http_client::HttpClient;
use anyhow::{bail, Result};

pub const PROFILE_URL: &str = "https://learning.oreilly.com/profile/";

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
