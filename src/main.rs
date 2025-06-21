use anyhow::{Context, Result};
use clap::Parser;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashSet, fs, time::Instant};
use tree_sitter_typescript::LANGUAGE_TYPESCRIPT;

mod parsing;

/// A tool to check for typos in code.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// A glob path to the files to check, e.g. 'src/**/*.ts'
    path: String,
}

type Dictionary = HashSet<String>;

fn load_dictionaries(glob_path: &str) -> Result<Dictionary> {
    let files = glob::glob(glob_path).unwrap();

    let mut dictionary = HashSet::with_capacity(500_000);

    for file in files {
        let file = fs::read_to_string(file.context("Failed to read file")?)?;

        dictionary.extend(file.lines().map(|line| line.to_string()));
    }

    Ok(dictionary)
}

fn main() -> Result<()> {
    let args = Args::parse();

    let files = glob::glob(&args.path)
        .context("Failed to glob")?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    let dictionary = load_dictionaries("dictionaries/*")?;
    println!("{:?}", dictionary);

    let now = Instant::now();

    let result = files
        .par_iter()
        .map(|file| parsing::parser::parse_file(file, &LANGUAGE_TYPESCRIPT.into()));

    result.for_each(|_| {});

    println!("[*] Done with {} files in {:?}", files.len(), now.elapsed());

    Ok(())
}
