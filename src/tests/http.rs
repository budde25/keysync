use super::*;
use proptest::prelude::*;

/// Tests that we can get keys from a valid GitHub user
#[test]
#[ignore]
fn test_get_github_budde25() {
    let n = Network::new();
    let url = get_github("budde25");
    n.get_keys(&url).expect("Args are valid should return a result");
}

/// Tests that we can get keys from a valid GitLab user, using the default url
#[test]
#[ignore]
fn test_get_gitlab_budde25() {
    let n = Network::new();
    let url = get_gitlab("budde25", None);
    n.get_keys(&url).expect("Args are valid should return a result");
}

/// Tests that we can get keys from a valid GitHub user, using a custom url
#[test]
#[ignore]
fn test_get_wisc_gitlab_budd() {
    let n = Network::new();
    let url = get_gitlab(
        "budde25",
        Some(Url::parse("https://gitlab.cs.wisc.edu/").unwrap()),
    );
    n.get_keys(&url).expect("Args are valid should return a result");
}

/// Tests that we cannot get keys from a invalid GitHub user/url
#[test]
#[ignore]
fn test_get_invalid_url() {
    let n = Network::new();
    let url =
        get_gitlab("budde25", Some(Url::parse("https://abc.edu/").unwrap()));
    n.get_keys(&url).expect_err("Args not valid should not return result, 404");
}

/// Tests that we generate the correct usl for each service
#[test]
fn test_url_completion() {
    assert_eq!(&get_github("budde25"), "https://github.com/budde25.keys");
    assert_eq!(&get_gitlab("budde25", None), "https://gitlab.com/budde25.keys");
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

/// Tests that we can create all urls the the same time,
#[test]
fn test_create_urls_all() {
    let urls = create_urls("budde25", true, true, true, None);
    assert_eq!(urls.len(), 3);
}

/// Tests that we can pass no services, and a username and still return GitHub (our default)
#[test]
fn test_create_urls_none() {
    let urls = create_urls("budde25", false, false, false, None);
    assert_eq!(urls.len(), 1);
}

/// Tests that we can pass only github, and a username and still return GitHub (our default), should be the same test_create_urls_none()
#[test]
fn test_create_urls_only_github() {
    let urls = create_urls("budde25", true, false, false, None);
    assert_eq!(urls.len(), 1);
}

/// Tests the we can pass only launchpad and a username and it will return a Launchpad url and not also GitHub
#[test]
fn test_create_urls_only_launchpad() {
    let urls = create_urls("budde25", false, true, false, None);
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0], "https://launchpad.net/~budde25/+sshkeys");
}

/// Tests the if we pass a GitLab url but not GitLab bool, only github will still be create, this should be considered bad input regardless
#[test]
fn test_create_urls_no_gitlab_but_url() {
    let gitlab_url = Url::parse("https://gitlab.com").unwrap();
    let urls = create_urls("budde25", false, false, false, Some(gitlab_url));
    assert_eq!(urls.len(), 1);
    assert_eq!(urls[0], "https://github.com/budde25.keys");
}

// Tests that weird charecters don't crash url generation
proptest! {
    #[test]
    fn test_gen_produces_valid_url(s in "\\PC*") {
        let github = get_github(&s);
        Url::parse(&github).expect("URL should be valid");

        let launchpad = get_launchpad(&s);
        Url::parse(&launchpad).expect("URL should be valid");

        let gitlab = get_gitlab(&s, None);
        Url::parse(&gitlab).expect("URL should be valid");
    }
}
