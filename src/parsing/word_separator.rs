use fancy_regex::Regex;
use once_cell::sync::Lazy;
use unicode_segmentation::UnicodeSegmentation;

pub fn extract_words(text: &str) -> impl Iterator<Item = String> {
    text.split_whitespace()
        .flat_map(split_on_numbers)
        .flat_map(split_unicode_word_boundary)
        .flat_map(split_snake_case)
        .flat_map(split_camel_case)
        .map(|s| s.to_lowercase())
        .filter(|str| str.len() > 2)
}

fn split_unicode_word_boundary(text: &str) -> impl Iterator<Item = &str> {
    text.unicode_words()
}

fn split_snake_case(text: &str) -> impl Iterator<Item = &str> {
    text.split_terminator("_")
}

fn split_on_numbers(text: &str) -> impl Iterator<Item = &str> {
    text.split_terminator(|c| char::is_ascii_digit(&c))
}

fn split_camel_case(text: &str) -> impl Iterator<Item = &str> {
    static RE: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"(\p{Lu}+(?=\p{Lu}\p{Ll})|\p{Lu}?\p{Ll}+)").unwrap());

    RE.find_iter(text)
        .filter_map(|c| c.ok())
        .map(|c| c.as_str())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_split_white_space() {
        assert_eq!(
            extract_words("hello world").collect::<Vec<_>>(),
            ["hello", "world"]
        );
    }

    #[test]
    fn test_split_snake_case() {
        assert_eq!(
            extract_words("hello_world_test").collect::<Vec<_>>(),
            ["hello", "world", "test"]
        );
    }

    #[test]
    fn test_split_on_numbers() {
        assert_eq!(
            extract_words("hello2world").collect::<Vec<_>>(),
            ["hello", "world"]
        );
    }

    #[test]
    fn test_split_camel_case() {
        assert_eq!(
            extract_words("camelCaseTest").collect::<Vec<_>>(),
            ["camel", "case", "test"]
        );
    }

    #[test]
    fn test_function_definition() {
        assert_eq!(
            extract_words("function parseJson(text: string)").collect::<Vec<_>>(),
            ["function", "parse", "json", "text", "string"]
        );
    }

    #[test]
    fn discards_short_words_after_parsing() {
        assert_eq!(
            extract_words("fn isTheCatInTheDog").collect::<Vec<_>>(),
            ["the", "cat", "the", "dog"]
        )
    }
}
