use crate::cmd::{code, controller, r#enum};
use clap::{ArgMatches, Command};

#[allow(dead_code)]
pub fn command() -> ArgMatches {
    let matches = Command::new("pacman")
        .about("package manager utility")
        .version("5.2.1")
        .subcommand(
            Command::new("gen")
                .short_flag('G')
                .long_flag("gen")
                .about("gen.")
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(code::command())
                .subcommand(controller::command())
                .subcommand(r#enum::command()),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("gen", gen_matches)) => {
            match gen_matches.subcommand_matches("code") {
                Some(arg_matches) => {
                    code::execute(arg_matches);
                }
                _ => {}
            }
            match gen_matches.subcommand_matches("enum") {
                Some(arg_matches) => {
                    r#enum::execute(arg_matches);
                }
                _ => {}
            }
            match gen_matches.subcommand_matches("controller") {
                Some(arg_matches) => {
                    controller::execute(arg_matches);
                }
                _ => {}
            }
        }
        _ => {} // If all subcommands are defined above, anything else is unreachable
    }

    matches
}

#[allow(dead_code)]
pub(crate) fn out_file(arg_matches: &ArgMatches) -> String {
    let mut out_file = String::new();
    if let Some(out) = arg_matches.get_many::<String>("out") {
        out_file = out.map(|s| s.as_str()).collect::<Vec<_>>().join("");
    }
    out_file
}

#[allow(dead_code)]
pub(crate) fn name(arg_matches: &ArgMatches) -> String {
    let mut name = String::new();
    if let Some(out) = arg_matches.get_many::<String>("name") {
        name = out.map(|s| s.as_str()).collect::<Vec<_>>().join("");
    }
    name
}

#[allow(dead_code)]
pub(crate) fn file(arg_matches: &ArgMatches) -> String {
    let mut file = String::new();
    if let Some(out) = arg_matches.get_many::<String>("file") {
        file = out.map(|s| s.as_str()).collect::<Vec<_>>().join("");
    }
    file
}
