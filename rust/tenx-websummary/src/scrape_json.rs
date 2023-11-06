use std::io::Read;

use anyhow::{format_err, Error};
use itertools::Itertools;
use serde_json::Value;

const PREFIX: &str = "      const data = ";
/// Tests are in tests/test_scrape.rs
pub fn scrape_json_str_from_html<R: Read>(mut reader: R) -> Result<String, Error> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    let buf = String::from_utf8(buf).unwrap();
    // Could use a crate like scraper or html5ever for html parsing instead of this simple approach
    // but that brings in a number of dependencies
    Ok(buf
        .lines()
        .filter_map(|line| line.strip_prefix(PREFIX))
        .exactly_one()
        .map_err(|e| format_err!("{e}"))?
        .to_string())
}

pub fn scrape_json_from_html<R: Read>(reader: R) -> Result<Value, Error> {
    Ok(serde_json::from_str(&scrape_json_str_from_html(reader)?)?)
}
