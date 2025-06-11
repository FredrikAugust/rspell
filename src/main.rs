use anyhow::{Context, Result};
use clap::Parser;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{collections::HashSet, fs, path::PathBuf, time::Instant};
use tree_sitter::Parser as TSParser;
use tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
use unicode_segmentation::UnicodeSegmentation;

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

    let dictionary = load_dictionaries("src/dictionaries/*")?;

    let now = Instant::now();

    let mut found = 0;
    let mut total = 0;

    let results = files.par_iter().filter_map(|file| {
        let mut parser = TSParser::new();
        parser
            .set_language(&LANGUAGE_TYPESCRIPT.into())
            .context("Failed to set language")
            .ok()?;

        Some(handle_file(file.to_owned(), &dictionary, parser))
    });

    for (found_in_file, total_in_file) in results
        .collect::<Vec<_>>()
        .into_iter()
        .filter_map(Result::ok)
    {
        found += found_in_file;
        total += total_in_file;
    }

    println!("[*] Done with {} files in {:?}", files.len(), now.elapsed());
    println!("[*] Found {} typos in {} words", found, total);

    Ok(())
}

const KIND_TO_TYPE_CHECK: &[&str] = &[
    "comment",
    "string_fragment",
    "identifier",
    "property_identifier",
];

fn handle_file(file: PathBuf, dictionary: &Dictionary, mut parser: TSParser) -> Result<(i32, i32)> {
    let content = fs::read_to_string(&file).context("Failed to read file")?;

    let tree = parser
        .parse(&content, None)
        .context("Failed to parse file")?;

    let mut found = 0;
    let mut total = 0;

    let mut cursor = tree.root_node().walk();

    loop {
        let node = cursor.node();
        let kind = node.kind();

        let byte_range = node.byte_range();

        let text = &content[byte_range];

        if KIND_TO_TYPE_CHECK.contains(&kind) {
            if contains_typo(text, dictionary) {
                found += 1;
            }
            total += 1;
        }

        if cursor.goto_first_child() {
            continue;
        }

        if cursor.goto_next_sibling() {
            continue;
        }

        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return Ok((found, total));
            }
        }
    }
}

fn contains_typo(input: &str, dictionary: &Dictionary) -> bool {
    if input.len() < 3 {
        return false;
    }

    if dictionary.contains(input) {
        return false;
    }

    let word_parts = get_all_word_parts(input);

    if word_parts.len() == 1 {
        return true;
    }

    for word in word_parts {
        if contains_typo(&word.to_lowercase(), dictionary) {
            return true;
        }
    }

    false
}

fn get_all_word_parts(input: &str) -> Vec<String> {
    let word_parts = split_special_characters(input);
    word_parts.into_iter().flat_map(split_pascal_case).collect()
}

fn split_special_characters(input: &str) -> impl Iterator<Item = &str> {
    input.unicode_words()
}

fn split_pascal_case(input: &str) -> Vec<String> {
    let mut result = Vec::new();

    let mut current = String::new();

    for c in input.chars() {
        if c.is_uppercase() && !current.is_empty() {
            result.push(current);
            current = String::new();
        }

        current.push(c);
    }

    result.push(current);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_special_characters() {
        assert_eq!(
            split_special_characters("Hello, world!").collect::<Vec<_>>(),
            vec!["Hello", "world"]
        );
    }

    #[test]
    fn test_split_special_characters_with_numbers() {
        assert_eq!(
            split_special_characters("/test/bin/bath").collect::<Vec<_>>(),
            vec!["test", "bin", "bath"]
        );
    }

    #[test]
    fn test_split_pascal_case() {
        assert_eq!(split_pascal_case("PascalCase"), vec!["Pascal", "Case"]);
    }

    #[test]
    fn test_split_pascal_case_with_numbers() {
        assert_eq!(
            split_pascal_case("PascalCase123"),
            vec!["Pascal", "Case123"]
        );
    }

    #[test]
    fn test_split_interface() {
        assert_eq!(
            split_pascal_case("IInterfaceCat"),
            vec!["I", "Interface", "Cat"]
        );
    }

    #[test]
    fn test_split_build_subject_crumb_trail() {
        assert_eq!(
            split_pascal_case("buildSubjectCrumbTrail"),
            vec!["build", "Subject", "Crumb", "Trail"]
        );
    }
}
