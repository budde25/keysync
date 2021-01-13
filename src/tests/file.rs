use super::*;
use assert_fs::prelude::*;

/// Tests that a new file will have no keys
#[test]
fn test_getting_empty_keys() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("authorized_keys"); // Don't create, functions should do it for us
    let authorized_keys = AuthorizedKeys::open_path(file.path()).unwrap();
    assert_eq!(authorized_keys.get_keys().unwrap().0.len(), 0);
}

/// Tests that a new file will have no keys
#[test]
fn test_getting_two_keys() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("authorized_keys");
    file.touch().unwrap();
    file.write_str("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDArNszFqR3vxzTe+pr/U/kmCn8aQAHNKfPMK4DJEvMvEbypiJV3Pm4iQG8jK6xBOTvcrFJTDX0VvgG0ky+iGOaLXw/M30BUsRhZlonasa0tbuu1PtHXlToXaCPyIPB39XucTjOQYtyFoS7yMfBuw0JhQ4ETJflvvHet5UkrbcqoSrac2ljtokmwR7z6cFEJTDXncEAhJsSJVQgPXWlf/j76XV8tP7ZFOBR7UVLSR2TXCLtg67o4Whu3ji/BV5Qa6t6Ef6rT4mndB29rY9D35qpASVlic84WzYKwRSfsc9FtryaA6mQMbfhN3xySKkfV5CgrVCH/rHGP09VzMlrlR+tHZDqznxeL4pr7+uJOHvMbgZHBvdbanQyApSGdB6HbRB1z8lVmbtOAsuK4TNkTQUNo8204NKJgtEsZnbqOWM0OMiJpjmhftqMq0Wl7OzZYWDzAEgS3ELoAl1DCkO4RkXsXWdHNK3p2MtxXOj3yM6MWZTPGT3dJXqATdu4lzknvSc= #maybe a comment
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDPx4jsUuivW/Yz0r7eD/InptzObq+qmwEP7fJrNZIOkKYyfVaxIxHYnAix7h4Qjk6dRq15to9slBSohRlXpXAx0WFpOMRgxC56uqnbGfh3fh8XxEIr23OHxiwoh4paS6CKu9Jz53S8lM6jSHsdH+0CmLm/iEw9Y0KtzOEzee6RR6EJUvs4TGSvaapOQJse4ZQNFJU0xBMVaGs4HQ2VitwrWVn/lvJoSoWk2fAAEGLcI2FOEoMBfnaAwyRj3F/L3hJ4vu77N7qvxdVCz7FRAEGPBcnoaeB4ivA2MXz3tEkHAilMTiUIMdPjS65lPyXfzWvlVQid3iMOb7oQcD4cI3oJ").unwrap();
    let authorized_keys = AuthorizedKeys::open_path(file.path()).unwrap();
    assert_eq!(authorized_keys.get_keys().unwrap().0.len(), 2);
}

/// Tests that we can write to an existing file
#[test]
fn test_writing_empty_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("authorized_keys");
    let keys = vec!["ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDArNszFqR3vxzTe+pr/U/kmCn8aQAHNKfPMK4DJEvMvEbypiJV3Pm4iQG8jK6xBOTvcrFJTDX0VvgG0ky+iGOaLXw/M30BUsRhZlonasa0tbuu1PtHXlToXaCPyIPB39XucTjOQYtyFoS7yMfBuw0JhQ4ETJflvvHet5UkrbcqoSrac2ljtokmwR7z6cFEJTDXncEAhJsSJVQgPXWlf/j76XV8tP7ZFOBR7UVLSR2TXCLtg67o4Whu3ji/BV5Qa6t6Ef6rT4mndB29rY9D35qpASVlic84WzYKwRSfsc9FtryaA6mQMbfhN3xySKkfV5CgrVCH/rHGP09VzMlrlR+tHZDqznxeL4pr7+uJOHvMbgZHBvdbanQyApSGdB6HbRB1z8lVmbtOAsuK4TNkTQUNo8204NKJgtEsZnbqOWM0OMiJpjmhftqMq0Wl7OzZYWDzAEgS3ELoAl1DCkO4RkXsXWdHNK3p2MtxXOj3yM6MWZTPGT3dJXqATdu4lzknvSc=".to_owned(),"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDPx4jsUuivW/Yz0r7eD/InptzObq+qmwEP7fJrNZIOkKYyfVaxIxHYnAix7h4Qjk6dRq15to9slBSohRlXpXAx0WFpOMRgxC56uqnbGfh3fh8XxEIr23OHxiwoh4paS6CKu9Jz53S8lM6jSHsdH+0CmLm/iEw9Y0KtzOEzee6RR6EJUvs4TGSvaapOQJse4ZQNFJU0xBMVaGs4HQ2VitwrWVn/lvJoSoWk2fAAEGLcI2FOEoMBfnaAwyRj3F/L3hJ4vu77N7qvxdVCz7FRAEGPBcnoaeB4ivA2MXz3tEkHAilMTiUIMdPjS65lPyXfzWvlVQid3iMOb7oQcD4cI3oJ".to_owned()];
    let authorized_keys = AuthorizedKeys::open_path(file.path()).unwrap();
    assert_eq!(authorized_keys.write_keys(keys, false).unwrap(), 2);
    assert_eq!(authorized_keys.get_keys().unwrap().0.len(), 2);
}

/// Tests that we can write to an existing file with data
#[test]
fn test_writing_empty_file_one_dup() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("authorized_keys");
    file.write_str("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDArNszFqR3vxzTe+pr/U/kmCn8aQAHNKfPMK4DJEvMvEbypiJV3Pm4iQG8jK6xBOTvcrFJTDX0VvgG0ky+iGOaLXw/M30BUsRhZlonasa0tbuu1PtHXlToXaCPyIPB39XucTjOQYtyFoS7yMfBuw0JhQ4ETJflvvHet5UkrbcqoSrac2ljtokmwR7z6cFEJTDXncEAhJsSJVQgPXWlf/j76XV8tP7ZFOBR7UVLSR2TXCLtg67o4Whu3ji/BV5Qa6t6Ef6rT4mndB29rY9D35qpASVlic84WzYKwRSfsc9FtryaA6mQMbfhN3xySKkfV5CgrVCH/rHGP09VzMlrlR+tHZDqznxeL4pr7+uJOHvMbgZHBvdbanQyApSGdB6HbRB1z8lVmbtOAsuK4TNkTQUNo8204NKJgtEsZnbqOWM0OMiJpjmhftqMq0Wl7OzZYWDzAEgS3ELoAl1DCkO4RkXsXWdHNK3p2MtxXOj3yM6MWZTPGT3dJXqATdu4lzknvSc= #maybe a comment
        ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDPx4jsUuivW/Yz0r7eD/InptzObq+qmwEP7fJrNZIOkKYyfVaxIxHYnAix7h4Qjk6dRq15to9slBSohRlXpXAx0WFpOMRgxC56uqnbGfh3fh8XxEIr23OHxiwoh4paS6CKu9Jz53S8lM6jSHsdH+0CmLm/iEw9Y0KtzOEzee6RR6EJUvs4TGSvaapOQJse4ZQNFJU0xBMVaGs4HQ2VitwrWVn/lvJoSoWk2fAAEGLcI2FOEoMBfnaAwyRj3F/L3hJ4vu77N7qvxdVCz7FRAEGPBcnoaeB4ivA2MXz3tEkHAilMTiUIMdPjS65lPyXfzWvlVQid3iMOb7oQcD4cI3oJ\n").unwrap();
    let keys = vec!["ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDNYBebNl8QY24FWJZLmwDrtSvQZaiMXe8sNUHbDNkPPNKe1XPqH1tD8ZBjqxlz/mxMfLOR6izFYj6Y6OSo4kqmuhhXZvSST0sPashF7PM/sfiR4gjoisuE/CfUsjGbvSBfQ5nfUJkQoUrzMQjA5CltbGO8prd6mt0u37RJqdBzAUAKaflwlpei5AWnE3a0RAlm29s/1bec28oQWkxqjLf11SqqJNgOsL70NDByMk+8LQ2OqEJHVHAIQjml/oSgKBG+SLO3uOX6M3bhmtAaP2MYAZdp/84qDFMX5lKk37k3xMjWmlHstyqkgJ3Hfp/Ft2KC1pPTKNK+C5VYqlROO/qKNSlIf80HNSGHBJymU/ncjcxOaofSkrLHzANsloYJileuUZs9sN48Eebrx0enz5Bf6ySxp5mmjesbASd0nxHXjZ0k9fiacAVeUTs54H/1VfIhLfVz4L8laZ79Ncbi0rprK7TofAKIErOOddgp3Wlj6CKeF+v6yPRCimb+IUt0qf0=".to_owned(),"ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQDPx4jsUuivW/Yz0r7eD/InptzObq+qmwEP7fJrNZIOkKYyfVaxIxHYnAix7h4Qjk6dRq15to9slBSohRlXpXAx0WFpOMRgxC56uqnbGfh3fh8XxEIr23OHxiwoh4paS6CKu9Jz53S8lM6jSHsdH+0CmLm/iEw9Y0KtzOEzee6RR6EJUvs4TGSvaapOQJse4ZQNFJU0xBMVaGs4HQ2VitwrWVn/lvJoSoWk2fAAEGLcI2FOEoMBfnaAwyRj3F/L3hJ4vu77N7qvxdVCz7FRAEGPBcnoaeB4ivA2MXz3tEkHAilMTiUIMdPjS65lPyXfzWvlVQid3iMOb7oQcD4cI3oJ".to_owned()];
    let authorized_keys = AuthorizedKeys::open_path(file.path()).unwrap();
    assert_eq!(authorized_keys.write_keys(keys, false).unwrap(), 1); // wrote one key only, since one was already there
    assert_eq!(authorized_keys.get_keys().unwrap().0.len(), 3);
}

/// Tests that we can write to an existing file with data
#[test]
fn test_writing_existing_with_a_newline() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("authorized_keys");
    file.write_str("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDArNszFqR3vxzTe+pr/U/kmCn8aQAHNKfPMK4DJEvMvEbypiJV3Pm4iQG8jK6xBOTvcrFJTDX0VvgG0ky+iGOaLXw/M30BUsRhZlonasa0tbuu1PtHXlToXaCPyIPB39XucTjOQYtyFoS7yMfBuw0JhQ4ETJflvvHet5UkrbcqoSrac2ljtokmwR7z6cFEJTDXncEAhJsSJVQgPXWlf/j76XV8tP7ZFOBR7UVLSR2TXCLtg67o4Whu3ji/BV5Qa6t6Ef6rT4mndB29rY9D35qpASVlic84WzYKwRSfsc9FtryaA6mQMbfhN3xySKkfV5CgrVCH/rHGP09VzMlrlR+tHZDqznxeL4pr7+uJOHvMbgZHBvdbanQyApSGdB6HbRB1z8lVmbtOAsuK4TNkTQUNo8204NKJgtEsZnbqOWM0OMiJpjmhftqMq0Wl7OzZYWDzAEgS3ELoAl1DCkO4RkXsXWdHNK3p2MtxXOj3yM6MWZTPGT3dJXqATdu4lzknvSc= #maybe a comment\n").unwrap();
    let keys = vec!["ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDNYBebNl8QY24FWJZLmwDrtSvQZaiMXe8sNUHbDNkPPNKe1XPqH1tD8ZBjqxlz/mxMfLOR6izFYj6Y6OSo4kqmuhhXZvSST0sPashF7PM/sfiR4gjoisuE/CfUsjGbvSBfQ5nfUJkQoUrzMQjA5CltbGO8prd6mt0u37RJqdBzAUAKaflwlpei5AWnE3a0RAlm29s/1bec28oQWkxqjLf11SqqJNgOsL70NDByMk+8LQ2OqEJHVHAIQjml/oSgKBG+SLO3uOX6M3bhmtAaP2MYAZdp/84qDFMX5lKk37k3xMjWmlHstyqkgJ3Hfp/Ft2KC1pPTKNK+C5VYqlROO/qKNSlIf80HNSGHBJymU/ncjcxOaofSkrLHzANsloYJileuUZs9sN48Eebrx0enz5Bf6ySxp5mmjesbASd0nxHXjZ0k9fiacAVeUTs54H/1VfIhLfVz4L8laZ79Ncbi0rprK7TofAKIErOOddgp3Wlj6CKeF+v6yPRCimb+IUt0qf0=".to_owned()];
    let authorized_keys = AuthorizedKeys::open_path(file.path()).unwrap();
    assert_eq!(authorized_keys.write_keys(keys, false).unwrap(), 1);
    assert_eq!(authorized_keys.get_keys().unwrap().0.len(), 2);
}

/// Tests that we can write to an existing file with data
#[test]
fn test_writing_existing_without_a_newline() {
    let temp = assert_fs::TempDir::new().unwrap();
    let file = temp.child("authorized_keys");
    file.write_str("ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDArNszFqR3vxzTe+pr/U/kmCn8aQAHNKfPMK4DJEvMvEbypiJV3Pm4iQG8jK6xBOTvcrFJTDX0VvgG0ky+iGOaLXw/M30BUsRhZlonasa0tbuu1PtHXlToXaCPyIPB39XucTjOQYtyFoS7yMfBuw0JhQ4ETJflvvHet5UkrbcqoSrac2ljtokmwR7z6cFEJTDXncEAhJsSJVQgPXWlf/j76XV8tP7ZFOBR7UVLSR2TXCLtg67o4Whu3ji/BV5Qa6t6Ef6rT4mndB29rY9D35qpASVlic84WzYKwRSfsc9FtryaA6mQMbfhN3xySKkfV5CgrVCH/rHGP09VzMlrlR+tHZDqznxeL4pr7+uJOHvMbgZHBvdbanQyApSGdB6HbRB1z8lVmbtOAsuK4TNkTQUNo8204NKJgtEsZnbqOWM0OMiJpjmhftqMq0Wl7OzZYWDzAEgS3ELoAl1DCkO4RkXsXWdHNK3p2MtxXOj3yM6MWZTPGT3dJXqATdu4lzknvSc= #maybe a comment").unwrap();
    let keys = vec!["ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDNYBebNl8QY24FWJZLmwDrtSvQZaiMXe8sNUHbDNkPPNKe1XPqH1tD8ZBjqxlz/mxMfLOR6izFYj6Y6OSo4kqmuhhXZvSST0sPashF7PM/sfiR4gjoisuE/CfUsjGbvSBfQ5nfUJkQoUrzMQjA5CltbGO8prd6mt0u37RJqdBzAUAKaflwlpei5AWnE3a0RAlm29s/1bec28oQWkxqjLf11SqqJNgOsL70NDByMk+8LQ2OqEJHVHAIQjml/oSgKBG+SLO3uOX6M3bhmtAaP2MYAZdp/84qDFMX5lKk37k3xMjWmlHstyqkgJ3Hfp/Ft2KC1pPTKNK+C5VYqlROO/qKNSlIf80HNSGHBJymU/ncjcxOaofSkrLHzANsloYJileuUZs9sN48Eebrx0enz5Bf6ySxp5mmjesbASd0nxHXjZ0k9fiacAVeUTs54H/1VfIhLfVz4L8laZ79Ncbi0rprK7TofAKIErOOddgp3Wlj6CKeF+v6yPRCimb+IUt0qf0=".to_owned()];
    let authorized_keys = AuthorizedKeys::open_path(file.path()).unwrap();
    assert_eq!(authorized_keys.write_keys(keys, false).unwrap(), 1);
    assert_eq!(authorized_keys.get_keys().unwrap().0.len(), 2);
}
