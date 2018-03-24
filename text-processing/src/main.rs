#![feature(box_syntax, nll, ascii_ctype, entry_and_modify)]

#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate futures;
extern crate indicatif;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate log;
extern crate rayon;
extern crate rust_stemmers;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod sparse_vector;
mod dictionary;
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
