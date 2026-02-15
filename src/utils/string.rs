
pub struct StringUtils;

impl StringUtils {
    pub fn generate_random_string(length: usize) -> String {
        use rand::{distr::Alphanumeric, Rng};
        rand::rng()
            .sample_iter(&Alphanumeric)
            .take(length)
            .map(char::from)
            .collect()
    }

    /// split by parts and add delimiter between them, e.g. "1234567890" with parts=3 becomes "123-456-7890"
    pub fn add_delimiter(s: &str, parts: usize) -> String {
        const DELIMITER: &str = "-";

        let part_length = (s.len() + parts as usize - 1) / parts as usize; // Round up division

        let parts: Vec<String> = s.chars()
            .collect::<Vec<char>>()
            .chunks(part_length)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect();

        parts.join(DELIMITER)
    }
}