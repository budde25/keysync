use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Error;
use reqwest::Response;
use std::time::Duration;
use url::Url;

const GITHUB_URL: &str = "https://github.com/";

fn get_client() -> Result<Client, Error> {
    let timeout = Duration::new(10, 0);
    return Ok(ClientBuilder::new().timeout(timeout).build()?)
}

#[tokio::main]
pub async fn get_github(username: &str) -> Result<String, Error> {
    let request: String = format!(
        "{url}{user}{ext}",
        url = GITHUB_URL,
        user = username,
        ext = ".keys"
    );

    let client: Client = get_client()?;

    let response: Result<Response, Error> = client
        .get(&request)
        .send()
        .await?
        .error_for_status();

    match response {
        Ok(resp) => return Ok(resp.text().await?),
        Err(e) => return Err(e),
    }
}

#[tokio::main]
pub async fn get_gitlab(username: &str, url: Url) -> Result<String, Error> {
    let request: String = format!(
        "{url}{user}{ext}",
        url = url,
        user = username,
        ext = ".keys"
    );

    let client: Client = get_client()?;

    let response: Result<Response, Error> = client
        .get(&request)
        .send()
        .await?
        .error_for_status();

    match response {
        Ok(resp) => return Ok(resp.text().await?),
        Err(e) => return Err(e),
    }
}