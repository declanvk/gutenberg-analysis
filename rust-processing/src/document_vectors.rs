use error::{Error, ErrorKind, Result};
use sparse_vector::SparseVector;

use clap::{App, Arg, ArgMatches, SubCommand};

use futures::executor::ThreadPoolBuilder;
use futures::stream::iter_result;
use futures::prelude::*;
use futures::channel::mpsc;

use rust_stemmers::{Algorithm, Stemmer};

use std::io::Read;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs::File;
use std::sync::Arc;

use serde_json;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b>
where
    'a: 'b,
{
    SubCommand::with_name("doc-vectors")
        .about("generate document vectors with dictionary")
        .version("0.1")
        .author("Declan Kelly. <dkelly.home@gmail.com>")
        .arg(
            Arg::with_name("texts")
                .short("t")
                .long("texts")
                .required(true)
                .value_name("texts-folder")
                .help("path of texts folder"),
        )
        .arg(
            Arg::with_name("dictionary")
                .short("d")
                .long("dictionary")
                .required(true)
                .value_name("dictionary-path")
                .help("path of dictionary file"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .required(true)
                .value_name("output-folder")
                .help("path of output folder"),
        )
}

pub fn execute_subcommand<'a>(matches: &ArgMatches<'a>) -> Result<()> {
    let output_path = value_t!(matches, "output", String)?;
    let texts_path_name = value_t!(matches, "texts", String)?;
    let dictionary_path_name = value_t!(matches, "dictionary", String)?;

    info!("Input texts folder: {:?}", texts_path_name);
    info!("Dictionary file: {:?}", dictionary_path_name);
    info!("Output folder: {:?}", output_path);

    let texts_path = Path::new(&texts_path_name);
    if !texts_path.exists() {
        return Err(ErrorKind::InvalidInput("Input texts folder does not exist.".into()).into());
    }

    let dictionary_path = Path::new(&dictionary_path_name);
    if !dictionary_path.exists() {
        return Err(ErrorKind::InvalidInput("Dictionary file does not exist.".into()).into());
    }

    if !Path::new(&output_path).exists() {
        return Err(ErrorKind::InvalidInput("Output folder does not exist".into()).into());
    }

    let dictionary = Arc::new(load_dictionary(dictionary_path)?);
    info!("Finished loading dictionary.");

    let en_stemmer = Arc::new(Stemmer::create(Algorithm::English));

    let (sender, reciever) = mpsc::channel::<PathBuf>(100);
    let texts_list = iter_result(texts_path.read_dir()?)
        .map_err(Error::from)
        .and_then(|p| Ok(p.path()))
        .forward(sender)
        .then(|_| Ok(()));

    let mut pool = ThreadPoolBuilder::new()
        .name_prefix("generate-dictionary")
        .after_start(|idx| info!("Thread {} starting pool", idx))
        .before_stop(|idx| info!("Thread {} stopping in pool", idx))
        .create();

    let stemmer_clone = Arc::clone(&en_stemmer);
    let dictionary_clone = Arc::clone(&dictionary);
    let generate_vectors = reciever
        .map_err(|n| n.never_into::<Error>())
        .and_then(|path| {
            let mut text: File = File::open(&path)?;
            let mut contents: String = String::new();

            text.read_to_string(&mut contents)?;

            let text_id: usize = path.file_stem().unwrap().to_str().unwrap().parse()?;
            info!("Text {}: read file contents", text_id);

            Ok((text_id, contents))
        })
        .and_then(move |(text_id, file_contents): (usize, String)| {
            let dictionary = &dictionary_clone;
            let stemmer = &stemmer_clone;

            info!("Text {}: cleaning file contents", text_id);
            let cleaned_text = file_contents.to_lowercase().replace(
                |c: char| !(c.is_ascii_alphabetic() || c.is_ascii_whitespace()),
                " ",
            );

            info!("Text {}: splitting and selecting words", text_id);
            let words = cleaned_text
                .split_whitespace()
                .map(|w| w.to_lowercase())
                .map(|w| stemmer.stem(&w).into_owned())
                .filter(|w| dictionary.contains_key(w))
                .filter(|w| w.len() > 2)
                .map(|w| (*dictionary.get(&w).unwrap(), 1));

            let mut vector: SparseVector<u32> = SparseVector::new(dictionary.len(), 0);

            info!("Text {}: creating word vector", text_id);
            vector.extend_with_merge(words, |old_value, new_value| {
                *old_value += new_value;
                0
            });

            Ok((text_id, vector))
        })
        .and_then(|(text_id, vector)| {
            info!("Text {}: creating or truncating output path", text_id);
            let output: &Path = output_path.as_ref();
            let output = File::create(output.join(format!("{}.json", text_id)))?;

            info!("Text {}: writing word vector", text_id);
            serde_json::to_writer(output, &vector)?;

            Ok(())
        })
        .collect::<Vec<_>>();

    pool.spawn(box texts_list).unwrap();
    let _: Vec<()> = pool.run(box generate_vectors)?;

    Ok(())
}

fn load_dictionary<P: AsRef<Path>>(dictionary_path: P) -> Result<HashMap<String, u32>> {
    let dictionary_file = File::open(dictionary_path.as_ref())?;
    let dictionary: Vec<(String, u32, u32)> = serde_json::from_reader(dictionary_file)?;

    Ok(dictionary
        .into_iter()
        .map(|(word, _, idx)| (word, idx))
        .collect())
}
