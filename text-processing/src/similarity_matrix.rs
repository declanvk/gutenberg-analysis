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

use rayon::prelude::*;
use rayon::iter::split;

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

    let whitelist = Arc::new(load_whitelist(whitelist_path)?);
    info!("Loaded whitelist");

    let all_document_vectors =
        load_all_document_vectors(Arc::clone(&whitelist), &document_vectors_path)?;
    info!("Loaded all document vectors");

    let mut sorted_whitelist: Vec<_> = whitelist.par_iter().cloned().collect();
    sorted_whitelist.par_sort();
    info!("Sorted whitelist");

    let cartesian_product = CartesianProduct {
        rows: &sorted_whitelist,
        columns: &sorted_whitelist,
        limit: 1000,
    };

    let all_pair_chunks: Vec<_> = split(cartesian_product, split_cartesian_product)
        .map(|cartesian| {
            iproduct!(
                cartesian.rows.iter().cloned(),
                cartesian.columns.iter().cloned()
            ).collect::<Vec<_>>()
        })
        .collect();
    info!(
        "Finished generating all {} whitelist chunks",
        all_pair_chunks.len()
    );

    let progress = ProgressBar::new(all_pair_chunks.len() as u64);
    for (idx, chunk) in all_pair_chunks.into_iter().enumerate() {
        let chunk_file = File::create(output_path.join(format!("chunk-{}.txt", idx)))?;

        let mut calculations = Vec::with_capacity(chunk.len());
        chunk
            .into_par_iter()
            .map(|(idx_a, idx_b)| {
                let document_a = &all_document_vectors[&idx_a];
                let document_b = &all_document_vectors[&idx_b];

                let similarity = cosine_similarity(document_a, document_b);

                (idx_a, idx_b, similarity)
            })
            .collect_into_vec(&mut calculations);

        serde_json::to_writer(chunk_file, &calculations)?;
        progress.tick();
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
) -> Result<HashMap<usize, SparseVector<u32>>> {
    let mut pool = ThreadPoolBuilder::new()
        .name_prefix("generate-dictionary")
        .after_start(|idx| info!("Thread {} starting in IO pool", idx))
        .before_stop(|idx| info!("Thread {} stopping in IO pool", idx))
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
            let document_vector: SparseVector<u32> = serde_json::from_reader(text)?;

            let text_id: usize = path.file_stem().unwrap().to_str().unwrap().parse()?;
            info!("Text {}: read file contents", text_id);

            Ok((text_id, document_vector))
        })
        .collect::<HashMap<usize, SparseVector<u32>>>();

    pool.run(document_vector_container)
}

fn cosine_similarity(vector_a: &SparseVector<u32>, vector_b: &SparseVector<u32>) -> f64 {
    let magnitude_a: u32 = vector_a.iter().map(|(_, x)| x * x).sum();
    let magnitude_b: u32 = vector_b.iter().map(|(_, x)| x * x).sum();

    let dot_product: u32 = vector_a
        .zip_nonzero_pairs(&vector_b)
        .map(|(a, b)| a * b)
        .sum();

    dot_product as f64 / (magnitude_a * magnitude_b) as f64
}

fn split_cartesian_product<'a, A: 'a + Clone>(
    mut product: CartesianProduct<A>,
) -> (CartesianProduct<A>, Option<CartesianProduct<A>>) {
    if product.rows.len() < product.limit && product.columns.len() < product.limit {
        info!(
            "{}x{} under size limit",
            product.rows.len(),
            product.columns.len()
        );

        (product, None)
    } else {
        info!("Splitting {}x{}", product.rows.len(), product.columns.len());
        if product.rows.len() > product.columns.len() {
            let midpoint = product.rows.len() / 2;
            let (left_rows, right_rows) = product.rows.split_at(midpoint);

            product.rows = left_rows;

            let new_product = CartesianProduct {
                rows: right_rows,
                columns: product.columns,
                limit: product.limit,
            };

            info!(
                "Into {}x{} and {}x{}",
                product.rows.len(),
                product.columns.len(),
                new_product.rows.len(),
                new_product.columns.len()
            );
            (product, Some(new_product))
        } else {
            let midpoint = product.columns.len() / 2;
            let (left_columns, right_columns) = product.columns.split_at(midpoint);

            product.columns = left_columns;

            let new_product = CartesianProduct {
                columns: right_columns,
                rows: product.rows,
                limit: product.limit,
            };

            info!(
                "Into {}x{} and {}x{}",
                product.rows.len(),
                product.columns.len(),
                new_product.rows.len(),
                new_product.columns.len()
            );
            (product, Some(new_product))
        }
    }
}

struct CartesianProduct<'a, A: 'a> {
    rows: &'a [A],
    columns: &'a [A],
    limit: usize,
}
