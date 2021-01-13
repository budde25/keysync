use clap::Shell;
use std::env;

include!("src/cli.rs");

fn main() {
    let outdir = env::var_os("CARGO_TARGET_DIR")
        .or_else(|| env::var_os("OUT_DIR"))
        .unwrap();

    let mut app = app();
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, &outdir);
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Zsh, &outdir);
    app.gen_completions(env!("CARGO_PKG_NAME"), Shell::Fish, &outdir);
}
