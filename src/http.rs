use super::util;
use anyhow::{anyhow, Context, Result};
use reqwest::{Client, ClientBuilder, Error, Response};
use std::time::Duration;
use std::vec;
use url::Url;

const GITHUB_URL: &str = "https://github.com/";
const GITLAB_URL: &str = "https://gitlab.com/";
const LAUNCHPAD_URL: &str = "https://launchpad.net/";

pub struct Network {
    client: Client,
}

impl Network {
    /// creates the client
    pub fn new() -> Self {
        let timeout = Duration::new(10, 0);
        Network {
            client: ClientBuilder::new().timeout(timeout).build().unwrap(),
        }
    }

    /// Gets the ssh keys from gitlab
    #[tokio::main]
    pub async fn get_keys<S: AsRef<str>>(&self, request_url: S) -> Result<Vec<String>> {
        let response: Result<Response, Error> = self
            .client
            .get(request_url.as_ref())
            .send()
            .await
            .with_context(|| format!("Error getting keys from: {}", request_url.as_ref()))?
            .error_for_status();

        match response {
            Ok(resp) => {
                let text = resp.text().await?;
                Ok(util::clean_keys(util::split_keys(&text)))
            }
            Err(e) => Err(anyhow!("{}", e)),
        }
    }

    /// Gets all the keys from the provided services
    pub fn get_keys_services<S: AsRef<str>>(
        &self,
        username: S,
        github: bool,
        launchpad: bool,
        gitlab: Option<Url>,
    ) -> Result<Vec<String>> {
        let mut all_keys: Vec<String> = vec![];
        let urls: Vec<String> = create_urls(username.as_ref(), github, launchpad, gitlab);
        for url in urls {
            let mut keys = self.get_keys(url)?;
            all_keys.append(&mut keys);
        }

        all_keys.sort();
        all_keys.dedup();
        Ok(all_keys)
    }
}

/// Returns a list of urls based for each service
pub fn create_urls(
    username: &str,
    github: bool,
    launchpad: bool,
    gitlab: Option<Url>,
) -> Vec<String> {
    // if none are selected default to github
    let real_github = if !github && !launchpad && gitlab.is_none() {
        true
    } else {
        false
    };

    let mut urls: Vec<String> = vec![];
    if real_github {
        urls.push(get_github(username))
    };
    if launchpad {
        urls.push(get_launchpad(username))
    };
    match gitlab {
        Some(url) => urls.push(get_gitlab(username, Some(url))),
        None => (),
    };
    urls
}

fn get_github(username: &str) -> String {
    format!("{}{}.keys", GITHUB_URL, username)
}

fn get_gitlab(username: &str, url: Option<Url>) -> String {
    match url {
        Some(u) => format!("{}{}.keys", u, username),
        None => format!("{}{}.keys", GITLAB_URL, username),
    }
}

fn get_launchpad(username: &str) -> String {
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
        let n = Network::new();
        let url = get_github("budde25");
        n.get_keys(&url)
            .expect("Args are valid should return a result");
    }

    #[test]
    #[ignore]
    fn get_gitlab_budde25() {
        let n = Network::new();
        let url = get_gitlab("budde25", None);
        n.get_keys(&url)
            .expect("Args are valid should return a result");
    }

    #[test]
    #[ignore]
    fn get_wisc_gitlab_budd() {
        let n = Network::new();
        let url = get_gitlab(
            "budde25",
            Some(Url::parse("https://gitlab.cs.wisc.edu/").unwrap()),
        );
        n.get_keys(&url)
            .expect("Args are valid should return a result");
    }

    #[test]
    #[ignore]
    fn get_invalid_url() {
        let n = Network::new();
        let url = get_gitlab("budde25", Some(Url::parse("https://abc.edu/").unwrap()));
        n.get_keys(&url)
            .expect("Args are valid should return a result");
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
