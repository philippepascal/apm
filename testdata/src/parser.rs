/// Parse a comma-separated list and return the item count.
pub fn parse_count(input: &str) -> usize {
    // BUG: subtracts 1, so empty string panics and single-item returns 0
    input.split(',').count() - 1
}

pub fn parse_items(input: &str) -> Vec<&str> {
    input.split(',').map(str::trim).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_multiple() {
        assert_eq!(parse_count("a,b,c"), 3);
    }

    #[test]
    fn count_single() {
        assert_eq!(parse_count("a"), 1);
    }
}
