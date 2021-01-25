<!-- Title -->
# Keysync

<!-- Subtitle -->
An SSH key syncing utility

<!-- Shields -->
![CI](https://github.com/budde25/ssh-key-sync/workflows/CI/badge.svg)
![CD](https://github.com/budde25/ssh-key-sync/workflows/CD/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/keysync)](https://crates.io/crates/keysync)
[![Crates.io](https://img.shields.io/crates/d/keysync)](https://crates.io/crates/keysync)

<!-- Table of Contents -->
<details>
  <summary><strong>Table of Contents</strong></summary>
  <ol>
    <li><a href="#about">About</a></li>
    <li><a href="#installation">Installation</a></li>
    <li>
      <a href="#usage">Usage</a>
      <ul>
        <li><a href="#examples">Examples</a></li>
      </ul>
    </li>
    <li>
      <a href="#building-and-testing">Building and Testing</a>
        <ul>
          <li><a href="#setup">Setup</a></li>
          <li><a href="#compile-and-run">Compile and Run</a></li>
          <li><a href="#testing">Testing</a></li>
          <li><a href="#documentation">Documentation</a></li>
        </ul>
    </li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#status">Status</a></li>
    <li><a href="#built-with">Built With</a></li>
    <li><a href="#contact">Contact</a></li>
    <li><a href="#license">License</a></li>
  </ol>
</details>

<!-- Image/GIF -->

<!-- Info -->
# About

keysync is a command line utility and service to help keep your local authorized_keys file synced to a master copy of public keys. 

The program allows syncing from Github, GitLab, Launchpad at the moment, custom url support is coming soon. It downloads and filters only keys that you don't already have a local copy of. This application can be used for either as one time sync when run, or running automatically in the background as a systemd service. You can have the file it updated at a preset interval or a custom cron expression, you can even support multiple users and providers.  

Note: Automatic jobs will fail if the computer goes to sleep/hibernate. The systemd daemon is recommended primarily for servers. For personal computers it is recommended to just run the command manually whenever you add public keys.

**Warning** If you're Github, GitLab or Launchpad is compromised an attacker can upload their keys to gain access to you're computer. For security please do not set this program up for a root user, you're just asking for trouble.

<!-- Installation -->
## Installation

If you have rust installed you can use cargo.  
requires a the following packages to be install:
libsqlite-dev, build-essential, libssl-dev (Debian names)  
`cargo install nxcloud`   
  
Other packaged binary's are available in [Releases](https://github.com/budde25/keysync/releases).  

<!-- Usage -->
## Usage

The binary name is `keysync`  

To display application use `keysync help`  
Use `keysync <subcommand> help` for help with that subcommand. 

<!-- Examples -->
### Examples

Downloads the public keys from github for the username.  
`keysync get <username>`  

Downloads the public keys from GitLab for the username, a url must be provided or '' for `https://gitlab.com`.  
`keysync get --gitlab <url> <username>`  

Adds automatic job for the user, where username is the Github or GitLab username.  
Valid schedules are [Hourly, Daily, Weekly, Monthly, Custom].  
`keysync set <username> <schedule>`  

 Adds automattic job for user with custom cron schedule.  
`keysync set <username> custom -c <cron>`  

<!-- Building and Testing -->
## Building and Testing

This repository is a standard rust project bin structure.  

<!-- Setup -->
### Setup

* Install [Rust](https://www.rust-lang.org/tools/install)  
* Install build-essential, libssl-dev, libsqlite3-dev  (Linux) <br> `apt install build-essential libssl-dev libsqlite3-dev` (Debian based)
* Clone repository

<!-- Compile and Run -->
### Compile and Run

Rust support building or running with the following commands:  
`cargo build` Will build an executable in `/target/debug/`.  
`cargo run -- <args>` Will build and run an executable.    

<!-- Testing -->
### Testing

Testing all standard test can be done with rust built in test framework.  
`cargo test`

Some tests cannot be completed with 100% reliability (for example they might fail without network access), this will run all ignored tests.  
`cargo test -- --ignored`

<!-- Docs -->
### Documentation

Rust built in documentation tools can be generated.  
`cargo doc`

To open with your default browser.  
`cargo doc --open`

<!-- Contributing -->
## Contributing

Contributions are completely welcome and encouraged!  
Examples of contributing could include: 

* Submitting a feature request or bug report.  
* Asking for improved documentation.  
* Code by creating a pull request.  

Refer to [Contributing](CONTRIBUTING.md)

<!-- Development Status -->
## Status

Development is still in progress with new features being planned.  
Feel free to [Contribute](#Contributing).

<!-- Technologies -->
## Built With

[Rust](https://www.rust-lang.org/)  
[SQLite](https://sqlite.org/index.html)  

<!-- Contact Info -->
## Contact

Created by [Ethan Budd](https://github.com/budde25)  
Email: [budde25@protonmail.com](mailto:budde25@protonmail.com)  

<!-- License -->
## License

Dual-licensed under either either of the following:
* [MIT License](LICENSE-MIT)
* [Apache License](LICENSE-APACHE)