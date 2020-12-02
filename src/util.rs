use anyhow::{anyhow, Result};
use nix::unistd::Uid;
use regex::Regex;
use rustyline::{error::ReadlineError, Editor};

// From regex example
macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

/// Filters the keys to prevent adding duplicates, also adds import comment
/// Returns a list of keys to that are unique
pub fn filter_keys(to_add: Vec<String>, exist: Vec<String>) -> Vec<String> {
    to_add
        .iter()
        .filter(|x| !exist.contains(x))
        .map(|x| x.to_owned() + " #ssh-import keysync")
        .collect()
}

/// Splits a string of keys into a list of keys based off a newline, also discards invalid keys
pub fn split_keys(all_keys: &str) -> Vec<String> {
    all_keys
        .split('\n')
        .map(|x| x.trim())
        .filter(|x| {
            let re: &Regex = regex!(
                r"^(ssh-rsa AAAAB3NzaC1yc2|ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNT|ecdsa-sha2-nistp384 AAAAE2VjZHNhLXNoYTItbmlzdHAzODQAAAAIbmlzdHAzOD|ecdsa-sha2-nistp521 AAAAE2VjZHNhLXNoYTItbmlzdHA1MjEAAAAIbmlzdHA1Mj|ssh-ed25519 AAAAC3NzaC1lZDI1NTE5|ssh-dss AAAAB3NzaC1kc3)",
            );
            re.is_match(x)
        })
        .map(|x| x.to_owned())
        .collect()
}

/// Removes any garbage from the keys Ex: comments
pub fn clean_keys(original_keys: Vec<String>) -> Vec<String> {
    original_keys
        .iter()
        .map(|x| x.split(' ').map(|x| x.to_owned()).collect::<Vec<String>>()[0..2].join(" "))
        .collect()
}

/// Runs the current command line options as root, (assuming sudo is installed)
pub fn run_as_root(user: Option<&str>) -> Result<()> {
    if !Uid::current().is_root() {
        let result = if let Some(u) = user {
            std::process::Command::new("sudo")
                .args(std::env::args())
                .arg("--user")
                .arg(u)
                .spawn()
        } else {
            std::process::Command::new("sudo")
                .args(std::env::args())
                .spawn()
        };

        match result {
            Ok(mut sudo) => {
                let output = sudo.wait().expect("Command failed to request root");
                if output.success() {
                    std::process::exit(0);
                } else {
                    Err(anyhow!("Command failed"))
                }
            }
            Err(_) => Err(anyhow!("Requires root")),
        }
    } else {
        Ok(())
    }
}

/// Prompts the user a question, with a (Y,n) attached.
/// Returns true if the user repsonds with y or yes, false otherwise
pub fn get_confirmation(query: &str) -> Result<bool> {
    let mut rl = Editor::<()>::new();
    let prompt = format!("{} (Y/n)\n>> ", query);
    let readline = rl.readline(&prompt);

    match readline {
        Ok(line) => {
            let clean_line = line.trim().to_lowercase();
            if clean_line == "y" || clean_line == "yes" {
                return Ok(true);
            }
        }
        Err(err) => match err {
            ReadlineError::Eof | ReadlineError::Interrupted => (),
            _ => println!("Error: {:?}", err),
        },
    }
    Ok(false)
}

pub fn get_current_user() -> Result<String> {
    Ok(std::env::var("USER")?)
}

// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;

    const SSH_KEY_TYPES: &'static str = r"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDFmp3Jxsec1EwRxUQgPjvpMyGrXsGm27UjUJpqN02FXW8cWe8qjzprQJ33cbCWEdwb1NcPW4I/oKjjuw7sY4dDs2ztDWFTOhFUD8yDp7Qk8lsn7Z72Im0gXfMq0hbB3Icbvi6t8zZbh+KYaklAfPNK5rJxopSmfkWzhAllEpSEGP6N09KTr7LT6cd0fm0QzyHSKW1ge2vPWVbQ71UOhidIcA17sHc2FsXkWR5XWxd6blqCZXSB8+S5UpdIn22lKl5tUqqBcHW+CjD89TUj7o/aD9Cq8BcKvlUyaRZSRrfTGTTySwNBiBRHqsAReFqPG0YkBt7X84SI6QKMbuSYU7VGhVsVak0cgB5ZSKrwGYrRvGnsJEHl43m4l14DJVdzXxZ/ol1CO5B9PmBPX62rgdKMEHfgL+e6Tm3sXd8uxKivikAHDsJyxfeaJN6U/WSqT9YgU+cRHvCFCESAq9nvn/jbX50Xxu1LLH/LAzAqhEujADJ1xOsZCCSizX+4ipiHm5LlDkYxP+4muDMb1rLPmS+/kqEXpNiJCdhDQQqDdElv+s4dy4+zzjP5jfyqifzJ7CxOgMyMq5WaPksF21mQiFKdz98ZLsMDFqwGrcXwjPiu+5pxhMuVjPiyOg69zpWpmSUcPDk4AcCaP6LF7hMp6//rCyxe1Clte7om0XZnTLwTAw== #Has a comment
                                        ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAICRDxyCYqJ/4RK8qJolhgsD5hb11ChbKpkHmB2rHUxGf budd@io
                                        ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBEX4kQM8rfCj7PWvDTVbhWDcJfi8FAMZan5+ymQh3hcyjJasXzOr3gZXbXikmt18nEzimABjGCaDN77SSmw+6RE= budd@io
                                        ecdsa-sha2-nistp384 AAAAE2VjZHNhLXNoYTItbmlzdHAzODQAAAAIbmlzdHAzODQAAABhBCN9rPtgo3xWgWleJ5D3yPBNB+VEgB8N9AvYI96XfOBeM4RF8rvXWQwsfa2JV0KQInxdFBfGvtosAtKVIFZaxDdfBfEM5iQApuEHrZuQlYkurTV405X7SDqyMMRwXubiQg==
                                        ecdsa-sha2-nistp521 AAAAE2VjZHNhLXNoYTItbmlzdHA1MjEAAAAIbmlzdHA1MjEAAACFBAHHRe0RvfqX73blygXzMpIaDX+3xXxJbMBeRgBWt2EkJ25Fhu9CWaZBrp0txHlJQLZS5xcO7MsTnonUE4bkyCkACQEJl6+Ijo/jn0QuL6GAZA0jyHUTdsBKVnjoppXt6G0dQELzh341IeIVHW6Fgvxc6j0IPGnNZyS59BusC0hQCrlNJQ==";

    /// Tests that clean keys removes the junk off the end of keys
    #[test]
    fn test_keys_clean() {
        let keys = vec![
            r"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDFmp3Jxsec1EwRxUQgPjvpMyGrXsGm27UjUJpqN02FXW8cWe8qjzprQJ33cbCWEdwb1NcPW4I/oKjjuw7sY4dDs2ztDWFTOhFUD8yDp7Qk8lsn7Z72Im0gXfMq0hbB3Icbvi6t8zZbh+KYaklAfPNK5rJxopSmfkWzhAllEpSEGP6N09KTr7LT6cd0fm0QzyHSKW1ge2vPWVbQ71UOhidIcA17sHc2FsXkWR5XWxd6blqCZXSB8+S5UpdIn22lKl5tUqqBcHW+CjD89TUj7o/aD9Cq8BcKvlUyaRZSRrfTGTTySwNBiBRHqsAReFqPG0YkBt7X84SI6QKMbuSYU7VGhVsVak0cgB5ZSKrwGYrRvGnsJEHl43m4l14DJVdzXxZ/ol1CO5B9PmBPX62rgdKMEHfgL+e6Tm3sXd8uxKivikAHDsJyxfeaJN6U/WSqT9YgU+cRHvCFCESAq9nvn/jbX50Xxu1LLH/LAzAqhEujADJ1xOsZCCSizX+4ipiHm5LlDkYxP+4muDMb1rLPmS+/kqEXpNiJCdhDQQqDdElv+s4dy4+zzjP5jfyqifzJ7CxOgMyMq5WaPksF21mQiFKdz98ZLsMDFqwGrcXwjPiu+5pxhMuVjPiyOg69zpWpmSUcPDk4AcCaP6LF7hMp6//rCyxe1Clte7om0XZnTLwTAw== #Revmoes this comment".to_owned(),
            r"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDFmp3Jxsec1EwRxUQgPjvpMyGrXsGm27UjUJpqN02FXW8cWe8qjzprQJ33cbCWEdwb1NcPW4I/oKjjuw7sY4dDs2ztDWFTOhFUD8yDp7Qk8lsn7Z72Im0gXfMq0hbB3Icbvi6t8zZbh+KYaklAfPNK5rJxopSmfkWzhAllEpSEGP6N09KTr7LT6cd0fm0QzyHSKW1ge2vPWVbQ71UOhidIcA17sHc2FsXkWR5XWxd6blqCZXSB8+S5UpdIn22lKl5tUqqBcHW+CjD89TUj7o/aD9Cq8BcKvlUyaRZSRrfTGTTySwNBiBRHqsAReFqPG0YkBt7X84SI6QKMbuSYU7VGhVsVak0cgB5ZSKrwGYrRvGnsJEHl43m4l14DJVdzXxZ/ol1CO5B9PmBPX62rgdKMEHfgL+e6Tm3sXd8uxKivikAHDsJyxfeaJN6U/WSqT9YgU+cRHvCFCESAq9nvn/jbX50Xxu1LLH/LAzAqhEujADJ1xOsZCCSizX+4ipiHm5LlDkYxP+4muDMb1rLPmS+/kqEXpNiJCdhDQQqDdElv+s4dy4+zzjP5jfyqifzJ7CxOgMyMq5WaPksF21mQiFKdz98ZLsMDFqwGrcXwjPiu+5pxhMuVjPiyOg69zpWpmSUcPDk4AcCaP6LF7hMp6//rCyxe1Clte7om0XZnTLwTAw== or any of this".to_owned(),
            r"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDFmp3Jxsec1EwRxUQgPjvpMyGrXsGm27UjUJpqN02FXW8cWe8qjzprQJ33cbCWEdwb1NcPW4I/oKjjuw7sY4dDs2ztDWFTOhFUD8yDp7Qk8lsn7Z72Im0gXfMq0hbB3Icbvi6t8zZbh+KYaklAfPNK5rJxopSmfkWzhAllEpSEGP6N09KTr7LT6cd0fm0QzyHSKW1ge2vPWVbQ71UOhidIcA17sHc2FsXkWR5XWxd6blqCZXSB8+S5UpdIn22lKl5tUqqBcHW+CjD89TUj7o/aD9Cq8BcKvlUyaRZSRrfTGTTySwNBiBRHqsAReFqPG0YkBt7X84SI6QKMbuSYU7VGhVsVak0cgB5ZSKrwGYrRvGnsJEHl43m4l14DJVdzXxZ/ol1CO5B9PmBPX62rgdKMEHfgL+e6Tm3sXd8uxKivikAHDsJyxfeaJN6U/WSqT9YgU+cRHvCFCESAq9nvn/jbX50Xxu1LLH/LAzAqhEujADJ1xOsZCCSizX+4ipiHm5LlDkYxP+4muDMb1rLPmS+/kqEXpNiJCdhDQQqDdElv+s4dy4+zzjP5jfyqifzJ7CxOgMyMq5WaPksF21mQiFKdz98ZLsMDFqwGrcXwjPiu+5pxhMuVjPiyOg69zpWpmSUcPDk4AcCaP6LF7hMp6//rCyxe1Clte7om0XZnTLwTAw==".to_owned(),
        ];
        let clean = clean_keys(keys);
        let clean_key = r"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDFmp3Jxsec1EwRxUQgPjvpMyGrXsGm27UjUJpqN02FXW8cWe8qjzprQJ33cbCWEdwb1NcPW4I/oKjjuw7sY4dDs2ztDWFTOhFUD8yDp7Qk8lsn7Z72Im0gXfMq0hbB3Icbvi6t8zZbh+KYaklAfPNK5rJxopSmfkWzhAllEpSEGP6N09KTr7LT6cd0fm0QzyHSKW1ge2vPWVbQ71UOhidIcA17sHc2FsXkWR5XWxd6blqCZXSB8+S5UpdIn22lKl5tUqqBcHW+CjD89TUj7o/aD9Cq8BcKvlUyaRZSRrfTGTTySwNBiBRHqsAReFqPG0YkBt7X84SI6QKMbuSYU7VGhVsVak0cgB5ZSKrwGYrRvGnsJEHl43m4l14DJVdzXxZ/ol1CO5B9PmBPX62rgdKMEHfgL+e6Tm3sXd8uxKivikAHDsJyxfeaJN6U/WSqT9YgU+cRHvCFCESAq9nvn/jbX50Xxu1LLH/LAzAqhEujADJ1xOsZCCSizX+4ipiHm5LlDkYxP+4muDMb1rLPmS+/kqEXpNiJCdhDQQqDdElv+s4dy4+zzjP5jfyqifzJ7CxOgMyMq5WaPksF21mQiFKdz98ZLsMDFqwGrcXwjPiu+5pxhMuVjPiyOg69zpWpmSUcPDk4AcCaP6LF7hMp6//rCyxe1Clte7om0XZnTLwTAw==".to_owned();
        for item in clean {
            assert_eq!(item, clean_key);
        }
    }

    /// Tests that split properly splits on the newline
    #[test]
    fn test_split() {
        let keys = r"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDFmp3Jxsec1EwRxUQgPjvpMyGrXsGm27UjUJpqN02FXW8cWe8qjzprQJ33cbCWEdwb1NcPW4I/oKjjuw7sY4dDs2ztDWFTOhFUD8yDp7Qk8lsn7Z72Im0gXfMq0hbB3Icbvi6t8zZbh+KYaklAfPNK5rJxopSmfkWzhAllEpSEGP6N09KTr7LT6cd0fm0QzyHSKW1ge2vPWVbQ71UOhidIcA17sHc2FsXkWR5XWxd6blqCZXSB8+S5UpdIn22lKl5tUqqBcHW+CjD89TUj7o/aD9Cq8BcKvlUyaRZSRrfTGTTySwNBiBRHqsAReFqPG0YkBt7X84SI6QKMbuSYU7VGhVsVak0cgB5ZSKrwGYrRvGnsJEHl43m4l14DJVdzXxZ/ol1CO5B9PmBPX62rgdKMEHfgL+e6Tm3sXd8uxKivikAHDsJyxfeaJN6U/WSqT9YgU+cRHvCFCESAq9nvn/jbX50Xxu1LLH/LAzAqhEujADJ1xOsZCCSizX+4ipiHm5LlDkYxP+4muDMb1rLPmS+/kqEXpNiJCdhDQQqDdElv+s4dy4+zzjP5jfyqifzJ7CxOgMyMq5WaPksF21mQiFKdz98ZLsMDFqwGrcXwjPiu+5pxhMuVjPiyOg69zpWpmSUcPDk4AcCaP6LF7hMp6//rCyxe1Clte7om0XZnTLwTAw==
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDArNszFqR3vxzTe+pr/U/kmCn8aQAHNKfPMK4DJEvMvEbypiJV3Pm4iQG8jK6xBOTvcrFJTDX0VvgG0ky+iGOaLXw/M30BUsRhZlonasa0tbuu1PtHXlToXaCPyIPB39XucTjOQYtyFoS7yMfBuw0JhQ4ETJflvvHet5UkrbcqoSrac2ljtokmwR7z6cFEJTDXncEAhJsSJVQgPXWlf/j76XV8tP7ZFOBR7UVLSR2TXCLtg67o4Whu3ji/BV5Qa6t6Ef6rT4mndB29rY9D35qpASVlic84WzYKwRSfsc9FtryaA6mQMbfhN3xySKkfV5CgrVCH/rHGP09VzMlrlR+tHZDqznxeL4pr7+uJOHvMbgZHBvdbanQyApSGdB6HbRB1z8lVmbtOAsuK4TNkTQUNo8204NKJgtEsZnbqOWM0OMiJpjmhftqMq0Wl7OzZYWDzAEgS3ELoAl1DCkO4RkXsXWdHNK3p2MtxXOj3yM6MWZTPGT3dJXqATdu4lzknvSc=
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDPx4jsUuivW/Yz0r7eD/InptzObq+qmwEP7fJrNZIOkKYyfVaxIxHYnAix7h4Qjk6dRq15to9slBSohRlXpXAx0WFpOMRgxC56uqnbGfh3fh8XxEIr23OHxiwoh4paS6CKu9Jz53S8lM6jSHsdH+0CmLm/iEw9Y0KtzOEzee6RR6EJUvs4TGSvaapOQJse4ZQNFJU0xBMVaGs4HQ2VitwrWVn/lvJoSoWk2fAAEGLcI2FOEoMBfnaAwyRj3F/L3hJ4vu77N7qvxdVCz7FRAEGPBcnoaeB4ivA2MXz3tEkHAilMTiUIMdPjS65lPyXfzWvlVQid3iMOb7oQcD4cI3oJ
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCvI/6E9JkXkkFO+ojRSwcaOXZEqUyV3zIJ8fQZp9pP2qd+cU5uveLceSTqMMyAV0lT4Qpuiq0WcD01GmYDv2yfjXnnSLEx9G+R/iEpkaYCl7rJyyKSA0dfcRrqPbiLGZ6d1UyqYNn+RgqtRJVqhyzNJMuFg8c2d+GrJaB76DgXFhbUL9Ju9V5KnwFF9x49tYNRICjl4zMmjefOH8gb1f8DF4ZbL3z7EjE/wdYfU8G7wy2IavbpApIhmKxFAPI3KYUddVHY+DrXsprqHjj4FsJ7dF9zn/tTByRFZiKhhnKBpTRXUunnw0Jn0XSkgTeBMT5z26gdwn034ukgt9uq/ZT5Epov1nhyarTfk5k6lv+5+EFvCHMM5ExcTTDGoMxOfGhujsb+0dtLTPDI33sRWtcBgbN9279sDPpJcdKLPpAk6v0tbBIuyyRR1S3YSjsvwtELgkddCaQzB3ZYFZLDmeeCC7zZnCjDwdkJua8hXNrUUhEzeScgFBJSeH8fw4dyQxM=
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQClA4WNL8+OkHPqn+DiL1o/gBGkJ2Qwo/18pfYnEteYLsunByBuFYhMd9i7GN3dMhwaDoPlF8oNQLGh9JFnhCGqqc36jTpahavnXNV6ZOCrtRdwh6MifjZmlqqJs+GMFm+iz3kXSTkl+uga91oHsg4pAy21n36ScxrUBfUimcd8/yCZBjLj2ZDCRnyQuIDUdgTkz6SIYPo5+bc6vxTPL29kKlR+AC44EwIfEgj7O0Vrtv2qe0ZEYCW2K0ZwD5dieKcV6uegu8qf6PaGPH01mwjbXdj8lG4EABpTswdCxIkJ5ax0dM6t79ZB3qZcBIPV++vKGOQ3M+jlUkTVEvHkBR0edcQmji1t7nbusOFT4wVTI3a/ykYE3w0p0+p5oQFsleiB9g751/DNLbd6mBfj1AMqKxVTPTnPcgzCPJWSN9mmRdUKX8DGn8ypdEsolU4YqOUuoJfuBoy0T55CJjFtFIOac1072GISqDhR2Eb21mORG+EjUZv9gSNi7Nh1VEQOIzc=
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCvfsMTAezNzrwVSCJkN8HfS9T0HFBO2tRfrnNmWQsvPwT+nuHE0Y+PvvuFpoTCRIqlecZHAFwuJGuSnkunWMQ+K2UvIjYRTLWDWyWoZoYxdN45tJNz8Hn6m6yUAldd6KgfgZJY6O9k5HNY2XhCXZNPdLXRV3VZ8rFGI59OK8lMzi7VtOf7VP+lmsEmvA15waGv2nUs/3jHzxLI3NfT4g8VZ8ViKVcRB/M/aD9/5aJ9B2PC+q92acpi0ahl46YffG9An1pQQ1v/qKQ61ARuCRga696/RlLotko9s7JlZ+LIzuj+LBjif/x44Uv9aTLof6kfPL5541qfMeyts/DmepD7
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDAsiuvGQ8YCQ6GA2Jpuk6WOMbfWEkN7FvD2eP/asJNlygFHZihmjwJfGuD5qx7Gj+gzLg+qS9pW4434Y8Xky43M4AyQmdpq440NlkFtnk3RMrqf/zD8NJXp3G3FywAEXRsLtJV5QpGQUlaMgjTxikVJcpav09BvOzwdqpv90rn6fUm8i7uB6vmQUvKPt9851tz73VJUtx0eX6DhC6o6mnIiVIkp3hhc/S9PyY5kRAnRXsjCuJ3IvyduFfSMYW3hKf764cg7jJpVmoc4yQQsb8JXTDq9LACr3YiYNWi7i9lODhQCl68FBJShdHATjLE19TveUtDl0n3F0PWt782bCqF1CnTahmnNNL5XooFwWt5cwn7KGxGzm0MRM6JU17qwxAwDW11PZejMpkGaHqefQMhCktJxxYKa1mixrQU97mtb/76EZEsOMmuI5eWcwcOQ5hgobdpJTyG6vJUonJq+a52uqknCxAeBXD6NgHGt3HZYyAJ5vCA+Gj6U1bzr114WM0=
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDRwE9FNjkzNdw2PpLCnHaqz+dDLkN+FTZMwQyLRcm5EVLShr28VllQoKIs4FvBO6VHPyr4R/YL4aVG0U42J4QPTU8ZgQfypf/91vHd39PyCuSNv96HkzkxIx+jOMVtDsDWGbeQ6WYqp6qTq+xLrHHAEyaRQMjNwtVMshC6mkzyqwWU3uyRzYasqf4Cq8/V6xLaREVolujGU92hUggA0H4PCzSpSMQDWhqqKUGgCEECwxqaM+tbq5emsE8x4Zgqz8GcUJO79w1HZSErJAwlQZE0qAtAJJ+6hf8K0mI31VaDvLPbmQgn4EXK45dj92imbkHTNQkuS7d1NEq3jcpSImhBjhpLbzMhfo77jubYCtS+PonFoVBtUquUuPgE8SVby2+wOfEjSZabomz/UnnCXGdQh5yb7E+8b1de91S6cla0fpRwMJmkl6n2Vwh819pnV+4XCGMZiR5f4z8qiu2k+gzBHs//SGIrwvdh9rOBuioJimw+1Xr0kAt6TY7GQN1coks=
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDNYBebNl8QY24FWJZLmwDrtSvQZaiMXe8sNUHbDNkPPNKe1XPqH1tD8ZBjqxlz/mxMfLOR6izFYj6Y6OSo4kqmuhhXZvSST0sPashF7PM/sfiR4gjoisuE/CfUsjGbvSBfQ5nfUJkQoUrzMQjA5CltbGO8prd6mt0u37RJqdBzAUAKaflwlpei5AWnE3a0RAlm29s/1bec28oQWkxqjLf11SqqJNgOsL70NDByMk+8LQ2OqEJHVHAIQjml/oSgKBG+SLO3uOX6M3bhmtAaP2MYAZdp/84qDFMX5lKk37k3xMjWmlHstyqkgJ3Hfp/Ft2KC1pPTKNK+C5VYqlROO/qKNSlIf80HNSGHBJymU/ncjcxOaofSkrLHzANsloYJileuUZs9sN48Eebrx0enz5Bf6ySxp5mmjesbASd0nxHXjZ0k9fiacAVeUTs54H/1VfIhLfVz4L8laZ79Ncbi0rprK7TofAKIErOOddgp3Wlj6CKeF+v6yPRCimb+IUt0qf0=";
        let arr = split_keys(keys);
        assert_eq!(arr.len(), 9);
    }

    /// Tests that the duplicate keys get filtered out
    #[test]
    fn test_filter() {
        let org_keys = r"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDFmp3Jxsec1EwRxUQgPjvpMyGrXsGm27UjUJpqN02FXW8cWe8qjzprQJ33cbCWEdwb1NcPW4I/oKjjuw7sY4dDs2ztDWFTOhFUD8yDp7Qk8lsn7Z72Im0gXfMq0hbB3Icbvi6t8zZbh+KYaklAfPNK5rJxopSmfkWzhAllEpSEGP6N09KTr7LT6cd0fm0QzyHSKW1ge2vPWVbQ71UOhidIcA17sHc2FsXkWR5XWxd6blqCZXSB8+S5UpdIn22lKl5tUqqBcHW+CjD89TUj7o/aD9Cq8BcKvlUyaRZSRrfTGTTySwNBiBRHqsAReFqPG0YkBt7X84SI6QKMbuSYU7VGhVsVak0cgB5ZSKrwGYrRvGnsJEHl43m4l14DJVdzXxZ/ol1CO5B9PmBPX62rgdKMEHfgL+e6Tm3sXd8uxKivikAHDsJyxfeaJN6U/WSqT9YgU+cRHvCFCESAq9nvn/jbX50Xxu1LLH/LAzAqhEujADJ1xOsZCCSizX+4ipiHm5LlDkYxP+4muDMb1rLPmS+/kqEXpNiJCdhDQQqDdElv+s4dy4+zzjP5jfyqifzJ7CxOgMyMq5WaPksF21mQiFKdz98ZLsMDFqwGrcXwjPiu+5pxhMuVjPiyOg69zpWpmSUcPDk4AcCaP6LF7hMp6//rCyxe1Clte7om0XZnTLwTAw==
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDArNszFqR3vxzTe+pr/U/kmCn8aQAHNKfPMK4DJEvMvEbypiJV3Pm4iQG8jK6xBOTvcrFJTDX0VvgG0ky+iGOaLXw/M30BUsRhZlonasa0tbuu1PtHXlToXaCPyIPB39XucTjOQYtyFoS7yMfBuw0JhQ4ETJflvvHet5UkrbcqoSrac2ljtokmwR7z6cFEJTDXncEAhJsSJVQgPXWlf/j76XV8tP7ZFOBR7UVLSR2TXCLtg67o4Whu3ji/BV5Qa6t6Ef6rT4mndB29rY9D35qpASVlic84WzYKwRSfsc9FtryaA6mQMbfhN3xySKkfV5CgrVCH/rHGP09VzMlrlR+tHZDqznxeL4pr7+uJOHvMbgZHBvdbanQyApSGdB6HbRB1z8lVmbtOAsuK4TNkTQUNo8204NKJgtEsZnbqOWM0OMiJpjmhftqMq0Wl7OzZYWDzAEgS3ELoAl1DCkO4RkXsXWdHNK3p2MtxXOj3yM6MWZTPGT3dJXqATdu4lzknvSc=
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDPx4jsUuivW/Yz0r7eD/InptzObq+qmwEP7fJrNZIOkKYyfVaxIxHYnAix7h4Qjk6dRq15to9slBSohRlXpXAx0WFpOMRgxC56uqnbGfh3fh8XxEIr23OHxiwoh4paS6CKu9Jz53S8lM6jSHsdH+0CmLm/iEw9Y0KtzOEzee6RR6EJUvs4TGSvaapOQJse4ZQNFJU0xBMVaGs4HQ2VitwrWVn/lvJoSoWk2fAAEGLcI2FOEoMBfnaAwyRj3F/L3hJ4vu77N7qvxdVCz7FRAEGPBcnoaeB4ivA2MXz3tEkHAilMTiUIMdPjS65lPyXfzWvlVQid3iMOb7oQcD4cI3oJ
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQCvI/6E9JkXkkFO+ojRSwcaOXZEqUyV3zIJ8fQZp9pP2qd+cU5uveLceSTqMMyAV0lT4Qpuiq0WcD01GmYDv2yfjXnnSLEx9G+R/iEpkaYCl7rJyyKSA0dfcRrqPbiLGZ6d1UyqYNn+RgqtRJVqhyzNJMuFg8c2d+GrJaB76DgXFhbUL9Ju9V5KnwFF9x49tYNRICjl4zMmjefOH8gb1f8DF4ZbL3z7EjE/wdYfU8G7wy2IavbpApIhmKxFAPI3KYUddVHY+DrXsprqHjj4FsJ7dF9zn/tTByRFZiKhhnKBpTRXUunnw0Jn0XSkgTeBMT5z26gdwn034ukgt9uq/ZT5Epov1nhyarTfk5k6lv+5+EFvCHMM5ExcTTDGoMxOfGhujsb+0dtLTPDI33sRWtcBgbN9279sDPpJcdKLPpAk6v0tbBIuyyRR1S3YSjsvwtELgkddCaQzB3ZYFZLDmeeCC7zZnCjDwdkJua8hXNrUUhEzeScgFBJSeH8fw4dyQxM=
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQClA4WNL8+OkHPqn+DiL1o/gBGkJ2Qwo/18pfYnEteYLsunByBuFYhMd9i7GN3dMhwaDoPlF8oNQLGh9JFnhCGqqc36jTpahavnXNV6ZOCrtRdwh6MifjZmlqqJs+GMFm+iz3kXSTkl+uga91oHsg4pAy21n36ScxrUBfUimcd8/yCZBjLj2ZDCRnyQuIDUdgTkz6SIYPo5+bc6vxTPL29kKlR+AC44EwIfEgj7O0Vrtv2qe0ZEYCW2K0ZwD5dieKcV6uegu8qf6PaGPH01mwjbXdj8lG4EABpTswdCxIkJ5ax0dM6t79ZB3qZcBIPV++vKGOQ3M+jlUkTVEvHkBR0edcQmji1t7nbusOFT4wVTI3a/ykYE3w0p0+p5oQFsleiB9g751/DNLbd6mBfj1AMqKxVTPTnPcgzCPJWSN9mmRdUKX8DGn8ypdEsolU4YqOUuoJfuBoy0T55CJjFtFIOac1072GISqDhR2Eb21mORG+EjUZv9gSNi7Nh1VEQOIzc=";
        let new_keys = r"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQClA4WNL8+OkHPqn+DiL1o/gBGkJ2Qwo/18pfYnEteYLsunByBuFYhMd9i7GN3dMhwaDoPlF8oNQLGh9JFnhCGqqc36jTpahavnXNV6ZOCrtRdwh6MifjZmlqqJs+GMFm+iz3kXSTkl+uga91oHsg4pAy21n36ScxrUBfUimcd8/yCZBjLj2ZDCRnyQuIDUdgTkz6SIYPo5+bc6vxTPL29kKlR+AC44EwIfEgj7O0Vrtv2qe0ZEYCW2K0ZwD5dieKcV6uegu8qf6PaGPH01mwjbXdj8lG4EABpTswdCxIkJ5ax0dM6t79ZB3qZcBIPV++vKGOQ3M+jlUkTVEvHkBR0edcQmji1t7nbusOFT4wVTI3a/ykYE3w0p0+p5oQFsleiB9g751/DNLbd6mBfj1AMqKxVTPTnPcgzCPJWSN9mmRdUKX8DGn8ypdEsolU4YqOUuoJfuBoy0T55CJjFtFIOac1072GISqDhR2Eb21mORG+EjUZv9gSNi7Nh1VEQOIzc=
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQCvfsMTAezNzrwVSCJkN8HfS9T0HFBO2tRfrnNmWQsvPwT+nuHE0Y+PvvuFpoTCRIqlecZHAFwuJGuSnkunWMQ+K2UvIjYRTLWDWyWoZoYxdN45tJNz8Hn6m6yUAldd6KgfgZJY6O9k5HNY2XhCXZNPdLXRV3VZ8rFGI59OK8lMzi7VtOf7VP+lmsEmvA15waGv2nUs/3jHzxLI3NfT4g8VZ8ViKVcRB/M/aD9/5aJ9B2PC+q92acpi0ahl46YffG9An1pQQ1v/qKQ61ARuCRga696/RlLotko9s7JlZ+LIzuj+LBjif/x44Uv9aTLof6kfPL5541qfMeyts/DmepD7
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDAsiuvGQ8YCQ6GA2Jpuk6WOMbfWEkN7FvD2eP/asJNlygFHZihmjwJfGuD5qx7Gj+gzLg+qS9pW4434Y8Xky43M4AyQmdpq440NlkFtnk3RMrqf/zD8NJXp3G3FywAEXRsLtJV5QpGQUlaMgjTxikVJcpav09BvOzwdqpv90rn6fUm8i7uB6vmQUvKPt9851tz73VJUtx0eX6DhC6o6mnIiVIkp3hhc/S9PyY5kRAnRXsjCuJ3IvyduFfSMYW3hKf764cg7jJpVmoc4yQQsb8JXTDq9LACr3YiYNWi7i9lODhQCl68FBJShdHATjLE19TveUtDl0n3F0PWt782bCqF1CnTahmnNNL5XooFwWt5cwn7KGxGzm0MRM6JU17qwxAwDW11PZejMpkGaHqefQMhCktJxxYKa1mixrQU97mtb/76EZEsOMmuI5eWcwcOQ5hgobdpJTyG6vJUonJq+a52uqknCxAeBXD6NgHGt3HZYyAJ5vCA+Gj6U1bzr114WM0=
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDRwE9FNjkzNdw2PpLCnHaqz+dDLkN+FTZMwQyLRcm5EVLShr28VllQoKIs4FvBO6VHPyr4R/YL4aVG0U42J4QPTU8ZgQfypf/91vHd39PyCuSNv96HkzkxIx+jOMVtDsDWGbeQ6WYqp6qTq+xLrHHAEyaRQMjNwtVMshC6mkzyqwWU3uyRzYasqf4Cq8/V6xLaREVolujGU92hUggA0H4PCzSpSMQDWhqqKUGgCEECwxqaM+tbq5emsE8x4Zgqz8GcUJO79w1HZSErJAwlQZE0qAtAJJ+6hf8K0mI31VaDvLPbmQgn4EXK45dj92imbkHTNQkuS7d1NEq3jcpSImhBjhpLbzMhfo77jubYCtS+PonFoVBtUquUuPgE8SVby2+wOfEjSZabomz/UnnCXGdQh5yb7E+8b1de91S6cla0fpRwMJmkl6n2Vwh819pnV+4XCGMZiR5f4z8qiu2k+gzBHs//SGIrwvdh9rOBuioJimw+1Xr0kAt6TY7GQN1coks=
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDNYBebNl8QY24FWJZLmwDrtSvQZaiMXe8sNUHbDNkPPNKe1XPqH1tD8ZBjqxlz/mxMfLOR6izFYj6Y6OSo4kqmuhhXZvSST0sPashF7PM/sfiR4gjoisuE/CfUsjGbvSBfQ5nfUJkQoUrzMQjA5CltbGO8prd6mt0u37RJqdBzAUAKaflwlpei5AWnE3a0RAlm29s/1bec28oQWkxqjLf11SqqJNgOsL70NDByMk+8LQ2OqEJHVHAIQjml/oSgKBG+SLO3uOX6M3bhmtAaP2MYAZdp/84qDFMX5lKk37k3xMjWmlHstyqkgJ3Hfp/Ft2KC1pPTKNK+C5VYqlROO/qKNSlIf80HNSGHBJymU/ncjcxOaofSkrLHzANsloYJileuUZs9sN48Eebrx0enz5Bf6ySxp5mmjesbASd0nxHXjZ0k9fiacAVeUTs54H/1VfIhLfVz4L8laZ79Ncbi0rprK7TofAKIErOOddgp3Wlj6CKeF+v6yPRCimb+IUt0qf0=";
        let org_arr = split_keys(org_keys);
        let new_arr = split_keys(new_keys);
        let diff = filter_keys(org_arr, new_arr);
        assert_eq!(diff.len(), 4); // filters out 1 from the new (does not include any of the org)
    }

    /// Tests that only the one valid key here is taken by split
    #[test]
    fn test_split_bad_key() {
        let keys = "AAAAB3NzaC1yc2EAAAADAQABAAACAQDFmp3Jxsec1EwRxUQgPjvpMyGrXsGm27UjUJpqN02FXW8cWe8qjzprQJ33cbCWEdwb1NcPW4I/oKjjuw7sY4dDs2ztDWFTOhFUD8yDp7Qk8lsn7Z72Im0gXfMq0hbB3Icbvi6t8zZbh+KYaklAfPNK5rJxopSmfkWzhAllEpSEGP6N09KTr7LT6cd0fm0QzyHSKW1ge2vPWVbQ71UOhidIcA17sHc2FsXkWR5XWxd6blqCZXSB8+S5UpdIn22lKl5tUqqBcHW+CjD89TUj7o/aD9Cq8BcKvlUyaRZSRrfTGTTySwNBiBRHqsAReFqPG0YkBt7X84SI6QKMbuSYU7VGhVsVak0cgB5ZSKrwGYrRvGnsJEHl43m4l14DJVdzXxZ/ol1CO5B9PmBPX62rgdKMEHfgL+e6Tm3sXd8uxKivikAHDsJyxfeaJN6U/WSqT9YgU+cRHvCFCESAq9nvn/jbX50Xxu1LLH/LAzAqhEujADJ1xOsZCCSizX+4ipiHm5LlDkYxP+4muDMb1rLPmS+/kqEXpNiJCdhDQQqDdElv+s4dy4+zzjP5jfyqifzJ7CxOgMyMq5WaPksF21mQiFKdz98ZLsMDFqwGrcXwjPiu+5pxhMuVjPiyOg69zpWpmSUcPDk4AcCaP6LF7hMp6//rCyxe1Clte7om0XZnTLwTAw==
        ssh

        # comment here a 
        no hashtag
        edcdsa
        ssh this is a key";
        assert_eq!(split_keys(keys).len(), 0);
    }

    #[test]
    fn test_all_valid_keys() {
        assert_eq!(split_keys(SSH_KEY_TYPES).len(), 5);
    }
}
