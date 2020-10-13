# keysync - An SSH syncing utility and service

![CI/CD](https://github.com/budde25/ssh-key-sync/workflows/CI/CD/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/keysync)](https://crates.io/crates/keysync)
[![Crates.io](https://img.shields.io/crates/d/keysync)](https://crates.io/crates/keysync)

keysync is a command line utility and service to help keep your local authorized_keys file synced to a master copy of public keys. The program allows syncing from Github and Gitlab at the moment, custom url support is coming soon. It downloads and filters only keys that you don't already have a local copy of. This application can be used for either as one time sync when run, or running automatically in the background as a systemd service. You can have the file it updated at a preset interval or a custom cron expression, you can even support multiple users and providers.  

Note: Automatic jobs will fail if the computer goes to sleep/hibernate. The systemd daemon is recommeded primarily for servers. For personal computers it is recomended to just run the command manually whenever you add public keys.

**Warning** If you're Github or Gitlab is comprmised an attacker can upload their keys to gain access to you're computer. For security please do not set this program up for a root user, you're just asking for trouble.

## Install

Install deb from releases.  
`cargo install keysync` not reccomended, cannot run as daemon.  
More releases coming soon.  

## Usage

```
SSH Key Sync 0.1.4
A command line client and service for keeping SHH keys up to date with a list Ex: Github.

USAGE:
    keysync [FLAGS] <SUBCOMMAND>

FLAGS:
    -d, --dry-run    Runs the commands without commiting the changes
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Verbose mode (-v, -vv, -vvv)

SUBCOMMANDS:
    get     The username to fetch
    help    Prints this message or the help of the given subcommand(s)
    jobs    Current enabled jobs
    set     Add an automatic job
```

Use `keysync <subcommand> help` for help with that subcommand.
  
Examples:  
`keysync get <username>` Downloads the public keys from github for the username.  
`keysync get --gitlab <url> <username>` Downloads the public keys from gitlab for the username, a url must be provided or '' for `https://gitlab.com`.  
`keysync add <user> <username> <schedule>` Adds automatic job for the user, where username is the Github or Gitlab username.  
Valid schedules are [Hourly, Daily, Weekly, Monthly, Custom]  
`keysync add <user> <username> custom -c <cron>` Adds automactic job for user with custom cron schedule.  

## Setup

* Install [Rust](https://www.rust-lang.org/tools/install)  
* Clone repository

### Compile and Run

`cargo build` Will build an executable.  
`cargo run -- <args>` Will build and run an executable.  
`cargo doc` Will build the documentation.  

### Testing

`cargo test` Will run all the unit tests except for the ignored ones, ignored because they use network and won't pass 100% reliably.  
`cargo test -- --ignored` Will run all the tests, some may fail depending on server response time and your internet capabilities.  

## Built With

[Rust](https://www.rust-lang.org/)

## License

[GNU General Public License v3.0](https://github.com/budde25/ssh-key-sync/blob/master/LICENSE)  

## Author

Ethan Budd
