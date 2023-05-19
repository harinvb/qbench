/// Extracts multiple queries from a given string, separated by semicolons
///
/// # Examples
///
/// ```
/// let query_str = "SELECT * FROM users WHERE id = 1; SELECT * FROM orders WHERE user_id = 1;";
/// let queries = extract_multiline_queries(query_str);
/// assert_eq!(queries, vec!["SELECT * FROM users WHERE id = 1", "SELECT * FROM orders WHERE user_id = 1"]);
/// ```
///
/// ```
/// let query_str = "SELECT * FROM users WHERE id = 1";
/// let queries = extract_multiline_queries(query_str);
/// assert_eq!(queries, vec!["SELECT * FROM users WHERE id = 1"]);
/// ```
pub fn extract_multiline_queries(query_str: &str) -> Vec<&str> {
    // split the string using `split_inclusive` which will include the separator in the substring.
    // trim each substring to remove leading/trailing white spaces.
    // collect all the substrings as a vector of string slices.
    query_str.split_inclusive(';').map(|s| s.trim()).collect()
}
