use error::{Error, ErrorKind, Result};
use sparse_vector::SparseVector;

use clap::{App, Arg, ArgGroup, ArgMatches, SubCommand};

use futures::executor::ThreadPoolBuilder;
use futures::stream::iter_result;
use futures::prelude::*;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::fs::{DirBuilder, File};
use std::sync::Arc;
use std::collections::BinaryHeap;
use std::fs::remove_dir_all;

use rayon::prelude::*;

use serde_json;

use indicatif::ProgressBar;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b>
where
    'a: 'b,
{
    SubCommand::with_name("similarity")
        .about("generate document vectors with dictionary")
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
        .arg(
            Arg::with_name("chunk-size-limit")
                .short("l")
                .long("limit")
                .value_name("chunk-limit")
                .help("limit of output chunk size"),
        )
        .arg(
            Arg::with_name("num-chunks")
                .short("c")
                .long("chunks")
                .value_name("num-chunks")
                .help("number of chunks"),
        )
        .group(
            ArgGroup::with_name("chunks")
                .args(&["chunk-size-limit", "num-chunks"])
                .multiple(false)
                .required(true),
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

    if output_path.exists() {
        info!("Removing existing output directory");
        remove_dir_all(&output_path)?;
    }

    info!("Creating output folder recursively");
    DirBuilder::new().recursive(true).create(&output_path)?;

    info!("Loading whitelist");
    let whitelist = Arc::new(load_whitelist(whitelist_path)?);

    info!("Loading all document vectors");
    let all_document_vectors =
        load_all_document_vectors(Arc::clone(&whitelist), &document_vectors_path)?;

    let all_document_magnitudes: HashMap<usize, f32> = all_document_vectors
        .par_iter()
        .map(|(id, doc)| (*id, doc.magnitude()))
        .collect();

    info!("Sorting whitelist");
    let mut sorted_whitelist: Vec<_> = whitelist.par_iter().cloned().collect();
    sorted_whitelist.par_sort();

    info!("Generating whitelist chunks: ");
    let all_pair_chunks: Vec<_> = if matches.is_present("chunk-size-limit") {
        info!("Using maximum size restriction");
        let limit = value_t!(matches, "chunk-size-limit", usize)?;

        let cartesian_product = CartesianProduct {
            rows: &sorted_whitelist,
            columns: &sorted_whitelist,
            limit,
        };

        cartesian_product
            .split_until_base()
            .into_iter()
            .map(|cartesian| {
                iproduct!(
                    cartesian.rows.iter().cloned(),
                    cartesian.columns.iter().cloned()
                ).collect::<Vec<_>>()
            })
            .collect()
    } else {
        info!("Using fixed number of chunks");
        let num_chunks = value_t!(matches, "num-chunks", usize)?;

        let cartesian_product = CartesianProduct {
            rows: &sorted_whitelist,
            columns: &sorted_whitelist,
            limit: 1,
        };

        cartesian_product
            .split_until_num(num_chunks)
            .into_iter()
            .map(|cartesian| {
                iproduct!(
                    cartesian.rows.iter().cloned(),
                    cartesian.columns.iter().cloned()
                ).collect::<Vec<_>>()
            })
            .collect()
    };

    info!(
        "Splitting output matrix in {} chunks.",
        all_pair_chunks.len()
    );

    let similarity_progress = ProgressBar::new(all_pair_chunks.len() as u64);
    info!("Computing similarity for chunks and writing each to file");
    for (idx, chunk) in all_pair_chunks.into_iter().enumerate() {
        let chunk_file = File::create(output_path.join(format!("chunk-{}.txt", idx)))?;

        let mut calculations = Vec::with_capacity(chunk.len());
        chunk
            .into_par_iter()
            .map(|(idx_a, idx_b)| {
                if idx_a == idx_b {
                    (idx_a, idx_b, 1.0)
                } else {
                    let document_a = &all_document_vectors[&idx_a];
                    let document_b = &all_document_vectors[&idx_b];

                    let magnitude_a = &all_document_magnitudes[&idx_a];
                    let magnitude_b = &all_document_magnitudes[&idx_b];

                    let dot_product: f32 = document_a
                        .zip_nonzero_pairs(&document_b)
                        .map(|(a, b)| a * b)
                        .sum();

                    let similarity = dot_product / (magnitude_a * magnitude_b);

                    (idx_a, idx_b, similarity)
                }
            })
            .collect_into_vec(&mut calculations);

        serde_json::to_writer(chunk_file, &calculations)?;
        similarity_progress.inc(1);
    }

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
) -> Result<HashMap<usize, SparseVector<f32>>> {
    let mut pool = ThreadPoolBuilder::new()
        .name_prefix("generate-dictionary")
        .after_start(|idx| debug!("Thread {} starting in IO pool", idx))
        .before_stop(|idx| debug!("Thread {} stopping in IO pool", idx))
        .create();

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
            let document_vector: SparseVector<f32> = serde_json::from_reader(text)?;

            let text_id: usize = path.file_stem().unwrap().to_str().unwrap().parse()?;
            debug!("Text {}: read file contents", text_id);

            Ok((text_id, document_vector))
        })
        .collect::<HashMap<usize, SparseVector<f32>>>();

    pool.run(document_vector_container)
}

#[derive(Debug, PartialOrd, PartialEq, Eq, Ord)]
struct CartesianProduct<'a, A: 'a> {
    rows: &'a [A],
    columns: &'a [A],
    limit: usize,
}

impl<'a, A: 'a> CartesianProduct<'a, A>
where
    A: Ord,
{
    fn split_until_base(self) -> Vec<CartesianProduct<'a, A>> {
        let mut queue = BinaryHeap::new();
        let mut result = Vec::new();
        queue.push(self);

        while !queue.is_empty() {
            let mut elem = queue.pop().unwrap();

            let possible_child = elem.split();
            match possible_child {
                Some(new_child) => {
                    queue.push(elem);
                    queue.push(new_child);
                }
                None => result.push(elem),
            }
        }

        result
    }

    fn split_until_num(self, limit: usize) -> Vec<CartesianProduct<'a, A>> {
        let mut queue = BinaryHeap::new();
        let mut result = Vec::with_capacity(limit);
        queue.push(self);

        while result.len() < limit && !queue.is_empty() {
            let mut elem = queue.pop().unwrap();

            let possible_child = elem.split();
            match possible_child {
                Some(new_child) => {
                    queue.push(elem);
                    queue.push(new_child);
                }
                None => result.push(elem),
            }
        }

        result
    }

    fn size(&self) -> usize {
        self.rows.len() * self.columns.len()
    }

    fn split(&mut self) -> Option<Self> {
        if self.size() < self.limit {
            debug!(
                "{}x{} under size limit",
                self.rows.len(),
                self.columns.len()
            );

            None
        } else {
            debug!("Splitting {}x{}", self.rows.len(), self.columns.len());
            if self.rows.len() > self.columns.len() {
                let midpoint = self.rows.len() / 2;
                let (left_rows, right_rows) = self.rows.split_at(midpoint);

                self.rows = left_rows;

                let new_product = CartesianProduct {
                    rows: right_rows,
                    columns: self.columns,
                    limit: self.limit,
                };

                debug!(
                    "Into {}x{} and {}x{}",
                    self.rows.len(),
                    self.columns.len(),
                    new_product.rows.len(),
                    new_product.columns.len()
                );

                Some(new_product)
            } else {
                let midpoint = self.columns.len() / 2;
                let (left_columns, right_columns) = self.columns.split_at(midpoint);

                self.columns = left_columns;

                let new_product = CartesianProduct {
                    columns: right_columns,
                    rows: self.rows,
                    limit: self.limit,
                };

                debug!(
                    "Into {}x{} and {}x{}",
                    self.rows.len(),
                    self.columns.len(),
                    new_product.rows.len(),
                    new_product.columns.len()
                );
                Some(new_product)
            }
        }
    }
}
