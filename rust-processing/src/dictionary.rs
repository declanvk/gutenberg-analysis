use error::{Error, ErrorKind, Result};

use clap::{App, Arg, ArgMatches, SubCommand};

use futures::executor::ThreadPoolBuilder;
use futures::stream::iter_result;
use futures::prelude::*;
use futures::channel::mpsc;

use rust_stemmers::{Algorithm, Stemmer};

use std::io::Read;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::sync::Arc;

use serde_json;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b>
where
    'a: 'b,
{
    SubCommand::with_name("dictionary")
        .about("generate dictionary from text corpus")
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
            Arg::with_name("output")
                .short("o")
                .long("output")
                .required(true)
                .value_name("output-folder")
                .help("path of output folder"),
        )
        .arg(
            Arg::with_name("stopwords")
                .short("s")
                .long("stop")
                .required(true)
                .value_name("stopwords-file")
                .help("path of stopwords file"),
        )
}

pub fn execute_subcommand<'a>(matches: &ArgMatches<'a>) -> Result<()> {
    let output_path = value_t!(matches, "output", String)?;
    let texts_path_name = value_t!(matches, "texts", String)?;
    let stopwords_path = value_t!(matches, "stopwords", String)?;

    info!("Input texts folder: {:?}", texts_path_name);
    info!("Output folder: {:?}", output_path);
    info!("Stopwords file: {:?}", stopwords_path);

    let texts_path = Path::new(&texts_path_name);
    if !texts_path.exists() {
        return Err(ErrorKind::InvalidInput("Input texts folder does not exist.".into()).into());
    }

    if !Path::new(&stopwords_path).exists() {
        return Err(ErrorKind::InvalidInput("Stopwords file does not exist".into()).into());
    }

    let en_stemmer = Arc::new(Stemmer::create(Algorithm::English));

    let mut pool = ThreadPoolBuilder::new()
        .name_prefix("generate-dictionary")
        .after_start(|idx| info!("Thread {} starting pool", idx))
        .before_stop(|idx| info!("Thread {} stopping in pool", idx))
        .create()?;

    let (sender, reciever) = mpsc::channel::<PathBuf>(10);

    let stopwords = Arc::new(read_stopwords(stopwords_path)?);

    let texts_list = iter_result(texts_path.read_dir()?)
        .map_err(Error::from)
        .and_then(|p| Ok(p.path()))
        .forward(sender)
        .then(|_| Ok(()));

    let stemmer_copy = Arc::clone(&en_stemmer);
    let stopwords_copy = Arc::clone(&stopwords);

    let generate_dictionary = reciever
        .map_err(|n| n.never_into::<Error>())
        .and_then(|path| {
            info!("Processing {}", path.display());
            let mut text: File = File::open(path)?;
            let mut contents: String = String::new();

            text.read_to_string(&mut contents)?;

            Ok(contents)
        })
        .and_then(move |contents: String| {
            let stemmer = &stemmer_copy;
            let local_stopwords = &stopwords_copy;

            let words: HashMap<String, u32> = contents
                .to_lowercase()
                .replace(
                    |c: char| !(c.is_ascii_alphabetic() || c.is_ascii_whitespace()),
                    " ",
                )
                .split_whitespace()
                .map(|w| w.to_lowercase())
                .map(|w| stemmer.stem(&w).into_owned())
                .filter(|w| !local_stopwords.contains(w))
                .filter(|w| w.len() > 2)
                .map(|w| w.into())
                .fold(HashMap::new(), |mut acc, x| {
                    acc.entry(x).and_modify(|e| *e += 1).or_insert(1);

                    acc
                });

            Ok(words)
        })
        .fold(
            HashMap::new(),
            |mut acc: HashMap<String, u32>, x: HashMap<String, u32>| {
                for (word, count) in x.into_iter() {
                    acc.entry(word).and_modify(|e| *e += count).or_insert(count);
                }

                Ok(acc)
            },
        )
        .into_future()
        .and_then(|dictionary| {
            Ok(dictionary
                .into_iter()
                .enumerate()
                .map(|(idx, (word, count))| (word, count, idx))
                .collect::<Vec<_>>())
        });

    pool.spawn(box texts_list).unwrap();

    let dictionary: Vec<(String, u32, usize)> = pool.run(box generate_dictionary)?;

    info!("Creating or truncating output path.");
    let output = File::create(output_path)?;

    info!("Writing json dictionary.");
    serde_json::to_writer(output, &dictionary)?;

    Ok(())
}

fn read_stopwords(stopwords_path: String) -> Result<HashSet<String>> {
    let mut stopwords_file = File::open(stopwords_path)?;
    let mut file_contents = String::new();

    stopwords_file.read_to_string(&mut file_contents)?;

    Ok(file_contents.split_whitespace().map(From::from).collect())
}
