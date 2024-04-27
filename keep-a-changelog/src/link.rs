use std::fmt::Display;

use derive_getters::Getters;
use eyre::{OptionExt, Result};
use serde::Serialize;

#[derive(Debug, Clone, Getters, Serialize, PartialEq, Eq)]
pub struct Link {
    pub anchor: String,
    pub url: String,
}

impl Link {
    pub fn parse(line: String) -> Result<Self> {
        let mut parts = line.splitn(2, ": ").map(|s| s.to_string());
        let anchor = parts
            .next()
            .ok_or_eyre(format!("Missing anchor: {line}"))?
            .replace(['[', ']'], "");
        let url = parts.next().ok_or_eyre("Missing url")?;

        Ok(Self { anchor, url })
    }
}

impl Display for Link {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[{}]: {}", self.anchor, self.url)
    }
}
