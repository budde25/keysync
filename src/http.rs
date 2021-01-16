use anyhow::{anyhow, Context, Result};
use log::{debug, info};
use reqwest::{Client, ClientBuilder, Error, Response};
use std::time::Duration;
use std::vec;
use url::Url;

use super::util;

const GITHUB_URL: &str = "https://github.com/";
const GITLAB_URL: &str = "https://gitlab.com/";
const LAUNCHPAD_URL: &str = "https://launchpad.net/";

/// Network key request implementation
pub struct Network {
    client: Client,
}

impl Network {
    /// Creates the Network class, import since it is recommended to reuse the same client for all requests
    pub fn new() -> Self {
        let timeout = Duration::new(10, 0);
        let network = Network {
            client: ClientBuilder::new().timeout(timeout).build().unwrap(),
        };
        info!("Created Network object");
        network
    }

    /// Gets the SSH keys from a requested url (as string)
    /// Return a Vector of Strings that have been cleaned
    #[tokio::main]
    pub async fn get_keys<S: AsRef<str>>(
        &self,
        request_url: S,
    ) -> Result<Vec<String>> {
        let response: Result<Response, Error> = self
            .client
            .get(request_url.as_ref())
            .send()
            .await
            .with_context(|| {
                format!("Error getting keys from: {}", request_url.as_ref())
            })?
            .error_for_status();

        match response {
            Ok(resp) => {
                let text = resp.text().await?;
                let keys = util::clean_keys(util::split_keys(&text));
                debug!(
                    "Retrieved {} keys from {}",
                    keys.len(),
                    request_url.as_ref()
                );
                Ok(keys)
            }
            Err(e) => Err(anyhow!("{}", e)),
        }
    }

    /// Gets all the keys from the provided services, if none are selected it will still grab from Github
    pub fn get_keys_services<S: AsRef<str>>(
        &self,
        username: S,
        github: bool,
        launchpad: bool,
        gitlab: bool,
        gitlab_url: Option<Url>,
    ) -> Result<Vec<String>> {
        let mut all_keys: Vec<String> = vec![];
        let urls: Vec<String> = create_urls(
            username.as_ref(),
            github,
            launchpad,
            gitlab,
            gitlab_url,
        );
        for url in urls {
            let mut keys = self.get_keys(url)?;
            all_keys.append(&mut keys);
        }

        all_keys.sort();
        all_keys.dedup(); // Dedup ineffective without sorted keys
        info!("Retrieved {} unique keys", all_keys.len());
        Ok(all_keys)
    }
}

/// Returns a list of urls based for each service
pub fn create_urls(
    username: &str,
    github: bool,
    launchpad: bool,
    gitlab: bool,
    gitlab_url: Option<Url>,
) -> Vec<String> {
    // if none are selected default to GitHub
    let real_github = !github && !launchpad && !gitlab;
    debug!(
        "Creating URLS with username: {} for GitHub: {}, Launchpad: {}, Gitlab: {}, URL: {:?}",
        username, github, launchpad, gitlab, gitlab_url
    );

    let mut urls: Vec<String> = vec![];
    if real_github || github {
        urls.push(get_github(username))
    };
    if launchpad {
        urls.push(get_launchpad(username))
    };
    if gitlab {
        urls.push(get_gitlab(username, gitlab_url))
    };
    debug!("URLS that have been generated: {:?}", urls);
    urls
}

/// Creates a GitHub keys url with a username
fn get_github(username: &str) -> String {
    let url = format!("{}{}.keys", GITHUB_URL, username);
    debug!("GitHub URL: {}", url);
    url
}

/// Creates a GitLab keys urls with a username and url, if no url is provided it uses the default (https://gitlab.com)
fn get_gitlab(username: &str, url: Option<Url>) -> String {
    let url = match url {
        Some(u) => format!("{}{}.keys", u, username),
        None => format!("{}{}.keys", GITLAB_URL, username),
    };
    debug!("GitLab URL: {}", url);
    url
}

/// Creates a Launchpad keys url with a username
fn get_launchpad(username: &str) -> String {
    let url = format!("{}~{}/+sshkeys", LAUNCHPAD_URL, username);
    debug!("Launchpad URL: {}", url);
    url
}

// Unit Tests
#[cfg(test)]
#[path = "./tests/http.rs"]
mod test;
