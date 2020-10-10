use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Error;
use reqwest::Response;
use std::time::Duration;
use url::Url;

pub const GITHUB_URL: &str = "https://github.com/";
pub const GITLAB_URL: &str = "https://gitlab.com/";

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

// TESTS
#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    #[ignore]
    fn get_github_budde25() {
        let url = Url::parse(GITHUB_URL).unwrap();
        get_standard("budde25", url)
        .expect("Args are valid should return a result");
    }

    #[test]
    #[ignore]
    fn get_gitlab_budde25() {
        let url = Url::parse(GITLAB_URL).unwrap();
        get_standard("budde25", url)
        .expect("Args are valid should return a result");
    }

    #[test]
    #[ignore]
    fn get_wisc_gitlab_budd() {
        let url = Url::parse("https://gitlab.cs.wisc.edu").unwrap();
        get_standard("budd", url)
        .expect("Args are valid should return a result");
    }

    #[test]
    #[ignore]
    fn get_invalid_url() {
        let url = Url::parse("https://abc.edu").unwrap();
        get_standard("budd", url)
        .expect_err("Args are valid should not return a result");
    }
}