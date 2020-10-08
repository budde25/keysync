use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Error;
use reqwest::Response;
use std::time::Duration;
use url::Url;

pub const GITHUB_URL: &str = "https://github.com/";
pub const GITLAB_URL: &str = "https://gitlab.com)";

/// creates the client
fn get_client() -> Result<Client, Error> {
    let timeout = Duration::new(10, 0);
    return Ok(ClientBuilder::new().timeout(timeout).build()?);
}

/// Gets the ssh keys from gitlab
#[tokio::main]
pub async fn get_standard(username: &str, url: Url) -> Result<String, Error> {
    let request: String = format!(
        "{url}{user}{ext}",
        url = url,
        user = username,
        ext = ".keys"
    );

    let client: Client = get_client()?;

    let response: Result<Response, Error> = client.get(&request).send().await?.error_for_status();

    match response {
        Ok(resp) => return Ok(resp.text().await?),
        Err(e) => return Err(e),
    }
}
