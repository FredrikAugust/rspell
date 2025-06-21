use std::fs::read_to_string;
use std::path::PathBuf;

use anyhow::{Context, Result};
use tree_sitter::{Language, Node, Parser, Tree};

pub fn parse_file(path: &PathBuf, language: &Language) -> Result<()> {
    let mut parser = Parser::new();

    parser
        .set_language(language)
        .context("Could not set language on parser")?;

    let file_content = read_to_string(path).context("Could not read file")?;

    let tree = parser
        .parse(file_content.as_bytes(), None)
        .context("Could not parse file")?;

    traverse_tree(&tree, |node| {
        let text = node
            .utf8_text(file_content.as_bytes())
            .context("Could not get file content as utf8 string")?;

        println!("{:?}", text);

        Ok(())
    })
    .context("Could not traverse tree")?;

    Ok(())
}

pub fn traverse_tree<F>(tree: &Tree, visit: F) -> Result<()>
where
    F: Fn(Node) -> Result<()>,
{
    let mut cursor = tree.root_node().walk();

    loop {
        visit(cursor.node())?;

        if cursor.goto_first_child() {
            continue;
        }

        if cursor.goto_next_sibling() {
            continue;
        }

        while !cursor.goto_next_sibling() {
            if !cursor.goto_parent() {
                return Ok(());
            }
        }
    }
}
