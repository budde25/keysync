use clap::{App, AppSettings, Arg, SubCommand};

pub fn app() -> App<'static, 'static> {
    let settings = [
        AppSettings::ColoredHelp,
        AppSettings::InferSubcommands,
        AppSettings::VersionlessSubcommands,
    ];

    let get = SubCommand::with_name("get")
        .about("Retrieves a key from an online source")
        .setting(AppSettings::SubcommandRequiredElseHelp);
}
