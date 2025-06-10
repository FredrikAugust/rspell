use std::{collections::HashSet, fs, path::PathBuf};

use clap::Parser;

use tree_sitter::Parser as TSParser;
use tree_sitter_typescript::LANGUAGE_TYPESCRIPT;

/// A tool to check for typos in code.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// A glob path to the files to check, e.g. 'src/**/*.ts'
    path: String,
}

type Dictionary = HashSet<String>;

fn load_dictionaries(glob_path: &str) -> Dictionary {
    let files = glob::glob(glob_path).unwrap();

    let mut dictionary = HashSet::new();

    for file in files {
        let file = file.unwrap();
        let file = fs::read_to_string(file).unwrap();

        dictionary.extend(file.lines().map(|line| line.to_string()));
    }

    dictionary
}

fn main() {
    let args = Args::parse();

    let files = glob::glob(&args.path).unwrap();

    let mut parser = TSParser::new();
    parser.set_language(&LANGUAGE_TYPESCRIPT.into()).unwrap();

    let dictionary = load_dictionaries("src/dictionaries/*");

    for file in files {
        let file = file.unwrap();
        handle_file(file, &dictionary, &mut parser);
    }
}

const KIND_TO_TYPE_CHECK: &[&str] = &[
    "comment",
    "string_fragment",
    "identifier",
    "property_identifier",
];

fn handle_file(file: PathBuf, dictionary: &Dictionary, parser: &mut TSParser) {
    let content = fs::read_to_string(&file).unwrap();

    let tree = parser.parse(&content, None).unwrap();

    let mut found = 0;
    let mut total = 0;

    let mut cursor = tree.root_node().walk();

    loop {
        let node = cursor.node();
        let kind = node.kind();

        let byte_range = node.byte_range();

        let text = &content[byte_range.clone()];

        if KIND_TO_TYPE_CHECK.contains(&kind) {
            if contains_typo(&text, dictionary) {
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
                println!(
                    "{}: {} / {}",
                    file.file_name().unwrap().to_str().unwrap(),
                    found,
                    total
                );
                return;
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

    return false;
}

fn get_all_word_parts(input: &str) -> Vec<String> {
    let word_parts = split_special_characters(input);
    word_parts
        .into_iter()
        .map(|s| split_pascal_case(&s))
        .flatten()
        .collect()
}

fn split_special_characters(input: &str) -> Vec<String> {
    input
        .split(&['/', ' ', '\\', ' ', ',', '.', '!', '_', '-'])
        .filter_map(|s| {
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        })
        .collect()
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

    result.into_iter().map(|s| s).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_special_characters() {
        assert_eq!(
            split_special_characters("Hello, world!"),
            vec!["Hello", "world"]
        );
    }

    #[test]
    fn test_split_special_characters_with_numbers() {
        assert_eq!(
            split_special_characters("/test/bin/bath"),
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
