use colored::*;

use crate::context::Context;

pub fn print_checking_versions() {
    println!(
        "{}{}",
        "[rlx]: ".yellow().bold(),
        "Checking package versions...".blue().bold(),
    );
}

pub fn print_invalid_package_version(name: String, expected: String, actual: String) {
    eprintln!(
        "{}{}{}{}{}{}{}",
        "[rlx]: ".yellow().bold(),
        "Version of the ".red(),
        name.red().bold(),
        " is invalid, expected: ".red(),
        expected.red().bold(),
        ", actual: ".red(),
        actual.red().bold(),
    );
}

pub fn error(msg: &str) {
    eprintln!(
        "{}{}{}",
        "[rlx]: ".yellow().bold(),
        "Error: ".red(),
        msg.bold().red(),
    );
}

pub fn print_valid_package_version(name: String) {
    println!(
        "{}{}{}{}",
        "[rlx]: ".yellow().bold(),
        "Version of the ".green(),
        name.green().bold(),
        " is valid".green(),
    );
}

pub fn debug(ctx: &Context, msg: &str) {
    if !ctx.debug() {
        return;
    }
    println!("{}{}", "[DEBUG]: ".blue().bold(), msg);
}
