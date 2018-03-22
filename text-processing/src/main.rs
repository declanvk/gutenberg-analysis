#![feature(alloc, box_syntax, nll, ascii_ctype, entry_and_modify)]

extern crate alloc;
#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate futures;
#[macro_use]
extern crate log;
extern crate rust_stemmers;
extern crate serde_json;

mod sparse_vectors;
mod make_dictionary;
mod error;

use clap::AppSettings;

fn main() {
    env_logger::init();

    let matches = app_from_crate!()
        .settings(&[AppSettings::SubcommandRequiredElseHelp])
        .subcommand(make_dictionary::get_subcommand())
        .get_matches();

    let result = match matches.subcommand() {
        ("gen-dictionary", Some(args)) => make_dictionary::execute_subcommand(args),
        _ => Ok(()),
    };

    match result {
        Err(reason) => error!("Error! Reason: {}", reason),
        _ => {}
    }
}
