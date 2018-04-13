use error::{Error, ErrorKind, Result};
use sparse_vector::SparseVector;

use clap::{App, Arg, ArgMatches, SubCommand};

use futures::executor::ThreadPoolBuilder;
use futures::stream::iter_result;
use futures::prelude::*;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::fs::{DirBuilder, File};
use std::sync::Arc;
use std::f64::EPSILON;

use rayon::prelude::*;

use serde_json;

use indicatif::ProgressBar;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b>
where
    'a: 'b,
{
    SubCommand::with_name("tf-idf")
        .about("generate tf-idf weight document vectors from raw counts")
        .version("0.1")
        .author("Declan Kelly. <dkelly.home@gmail.com>")
        .arg(
            Arg::with_name("document-vectors")
                .short("d")
                .long("doc-vectors")
                .required(true)
                .value_name("document-vectors-folder")
                .help("path of document vectors folder"),
        )
        .arg(
            Arg::with_name("texts-whitelist")
                .short("w")
                .long("whitelist")
                .required(true)
                .value_name("whitelist-file")
                .help("path of texts whitelist file"),
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
    let output_path: PathBuf = value_t!(matches, "output", String)?.into();
    let document_vectors_path: PathBuf = value_t!(matches, "document-vectors", String)?.into();
    let whitelist_path: PathBuf = value_t!(matches, "texts-whitelist", String)?.into();

    info!("Input texts folder: {}", document_vectors_path.display());
    info!("Whitelist file: {}", whitelist_path.display());
    info!("Output folder: {}", output_path.display());

    if !document_vectors_path.exists() {
        return Err(
            ErrorKind::InvalidInput("Input document vectors folder does not exist.".into()).into(),
        );
    }

    if !whitelist_path.exists() {
        return Err(ErrorKind::InvalidInput("Texts whitelist file does not exist.".into()).into());
    }

    info!("Creating output folder recursively.");
    DirBuilder::new().recursive(true).create(&output_path)?;

    info!("Loading whitelist.");
    let whitelist = Arc::new(load_whitelist(whitelist_path)?);

    info!("Loading all document vectors");
    let all_document_vectors =
        load_all_document_vectors(Arc::clone(&whitelist), &document_vectors_path)?;

    let num_documents = all_document_vectors.len() as u32;

    let inverse_document_freq: HashMap<u32, u32> = all_document_vectors
        .par_iter()
        .map(|(_, document)| document.key_set())
        .fold(
            || HashMap::new(),
            |mut acc: HashMap<u32, u32>, elem| {
                for term in elem {
                    *acc.entry(term).or_insert(0) += 1;
                }

                acc
            },
        )
        .reduce(
            || HashMap::new(),
            |mut a: HashMap<u32, u32>, b: HashMap<u32, u32>| {
                for (key, value) in b {
                    *a.entry(key).or_insert(0) += value;
                }
                a
            },
        );

    let epsilon = 10f64 * EPSILON;
    let weighted_document_vectors: Vec<_> = all_document_vectors
        .into_par_iter()
        .map(|(text_id, document)| {
            let default = *document.default();
            let size = document.size();

            let new_vector_components = document
                .into_iter()
                .map(|(word_idx, raw_count)| {
                    let weighted_count = raw_count as f64
                        * (num_documents as f64 / inverse_document_freq[&word_idx] as f64).log10();
                    (word_idx, weighted_count)
                })
                .filter(|&(_, weighted_count)| {
                    weighted_count < -epsilon || epsilon < weighted_count
                });

            let mut new_vec = SparseVector::new(size, default as f64);

            new_vec.extend(new_vector_components);

            (text_id, new_vec)
        })
        .collect();

    let progress = ProgressBar::new(weighted_document_vectors.len() as u64);
    for (text_id, document) in weighted_document_vectors {
        debug!("Text {}: creating or truncating output path", text_id);
        let output: &Path = output_path.as_ref();
        let output = File::create(output.join(format!("{}.json", text_id)))?;

        debug!("Text {}: writing word vector", text_id);
        serde_json::to_writer(output, &document)?;
        progress.inc(1);
    }
    progress.finish_and_clear();

    Ok(())
}

fn load_whitelist<P: AsRef<Path>>(whitelist_path: P) -> Result<HashSet<usize>> {
    let mut whitelist_file = File::open(whitelist_path.as_ref())?;
    let mut file_contents = String::new();

    whitelist_file.read_to_string(&mut file_contents)?;

    Ok(file_contents
        .split_whitespace()
        .map(|p: &str| p.into())
        .map(|p: PathBuf| p.file_stem().unwrap().to_str().unwrap().parse().unwrap())
        .collect())
}

fn load_all_document_vectors(
    whitelist_copy: Arc<HashSet<usize>>,
    document_vectors_path: &Path,
) -> Result<HashMap<usize, SparseVector<u32>>> {
    let mut pool = ThreadPoolBuilder::new()
        .name_prefix("generate-dictionary")
        .after_start(|idx| debug!("Thread {} starting in IO pool", idx))
        .before_stop(|idx| debug!("Thread {} stopping in IO pool", idx))
        .create()?;

    let document_vectors_paths = iter_result(document_vectors_path.read_dir()?)
        .map_err(Error::from)
        .and_then(|p| Ok(p.path()))
        .filter(move |p: &PathBuf| {
            let document_id: usize = p.file_stem().unwrap().to_str().unwrap().parse().unwrap();
            Ok(whitelist_copy.contains(&document_id))
        });

    let document_vector_container = document_vectors_paths
        .and_then(|path: PathBuf| {
            let text: File = File::open(&path)?;
            let document_vector: SparseVector<u32> = serde_json::from_reader(text)?;

            let text_id: usize = path.file_stem().unwrap().to_str().unwrap().parse()?;
            debug!("Text {}: read file contents", text_id);

            Ok((text_id, document_vector))
        })
        .collect::<HashMap<_, _>>();

    pool.run(document_vector_container)
}
