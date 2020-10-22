use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Error;
use reqwest::Response;
use std::time::Duration;
use url::Url;

pub const GITHUB_URL: &str = "https://github.com/";
pub const GITLAB_URL: &str = "https://gitlab.com/";
pub const LAUNCHPAD_URL: &str = "https://launchpad.net/";

/// creates the client
fn get_client() -> Result<Client, Error> {
    let timeout = Duration::new(10, 0);
    Ok(ClientBuilder::new().timeout(timeout).build()?)
}

/// Gets the ssh keys from gitlab
#[tokio::main]
pub async fn get_keys(request_url: &str) -> Result<String, Error> {
    let client: Client = get_client()?;

    let response: Result<Response, Error> =
        client.get(request_url).send().await?.error_for_status();

    match response {
        Ok(resp) => Ok(resp.text().await?),
        Err(e) => Err(e),
    }
}

pub fn get_github(username: &str) -> String {
    format!("{}{}.keys", GITHUB_URL, username)
}

pub fn get_gitlab(username: &str, url: Option<Url>) -> String {
    match url {
        Some(u) => format!("{}{}.keys", u, username),
        None => format!("{}{}.keys", GITLAB_URL, username),
    }
}

pub fn get_launchpad(username: &str) -> String {
    format!("{}~{}/+sshkeys", LAUNCHPAD_URL, username)
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    #[ignore]
    fn get_github_budde25() {
        let url = get_github("budde25");
        get_keys(&url).expect("Args are valid should return a result");
    }

    #[test]
    #[ignore]
    fn get_gitlab_budde25() {
        let url = get_gitlab("budde25", None);
        get_keys(&url).expect("Args are valid should return a result");
    }

    #[test]
    #[ignore]
    fn get_wisc_gitlab_budd() {
        let url = get_gitlab(
            "budde25",
            Some(Url::parse("https://gitlab.cs.wisc.edu/").unwrap()),
        );
        get_keys(&url).expect("Args are valid should return a result");
    }

    #[test]
    #[ignore]
    fn get_invalid_url() {
        let url = get_gitlab("budde25", Some(Url::parse("https://abc.edu/").unwrap()));
        get_keys(&url).expect("Args are valid should return a result");
    }

    #[test]
    fn url_completion() {
        assert_eq!(&get_github("budde25"), "https://github.com/budde25.keys");
        assert_eq!(
            &get_gitlab("budde25", None),
            "https://gitlab.com/budde25.keys"
        );
        assert_eq!(
            &get_gitlab(
                "budde25",
                Some(Url::parse("https://gitlab.cs.wisc.edu/").unwrap())
            ),
            "https://gitlab.cs.wisc.edu/budde25.keys"
        );
        assert_eq!(
            &get_launchpad("budde25"),
            "https://launchpad.net/~budde25/+sshkeys"
        );
    }
}
