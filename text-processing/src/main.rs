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
extern crate num_traits;
extern crate rayon;
extern crate rust_stemmers;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod sparse_vector;
mod dictionary;
mod document_vectors;
mod similarity_matrix;
mod error;
mod tf_idf;

use clap::AppSettings;

fn main() {
    env_logger::init();

    let matches = app_from_crate!()
        .settings(&[AppSettings::SubcommandRequiredElseHelp])
        .subcommand(dictionary::get_subcommand())
        .subcommand(document_vectors::get_subcommand())
        .subcommand(similarity_matrix::get_subcommand())
        .subcommand(tf_idf::get_subcommand())
        .get_matches();

    let result = match matches.subcommand() {
        ("dictionary", Some(args)) => dictionary::execute_subcommand(args),
        ("doc-vectors", Some(args)) => document_vectors::execute_subcommand(args),
        ("similarity", Some(args)) => similarity_matrix::execute_subcommand(args),
        ("tf-idf", Some(args)) => tf_idf::execute_subcommand(args),
        _ => Ok(()),
    };

    match result {
        Err(reason) => error!("Error! Reason: {}", reason),
        _ => {}
    }
}
