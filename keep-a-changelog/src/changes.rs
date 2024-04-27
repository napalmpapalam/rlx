use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use eyre::{bail, Error};
use serde::{Deserialize, Serialize};

use crate::utils::substring;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Added,
    Changed,
    Deprecated,
    Removed,
    Fixed,
    Security,
}

impl FromStr for ChangeKind {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "added" => Ok(Self::Added),
            "changed" => Ok(Self::Changed),
            "deprecated" => Ok(Self::Deprecated),
            "removed" => Ok(Self::Removed),
            "fixed" => Ok(Self::Fixed),
            "security" => Ok(Self::Security),
            _ => bail!("Unknown change type: {}", s),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Changes {
    added: Vec<String>,
    changed: Vec<String>,
    deprecated: Vec<String>,
    removed: Vec<String>,
    fixed: Vec<String>,
    security: Vec<String>,
}

impl Changes {
    pub fn add(&mut self, kind: ChangeKind, change: String) {
        match kind {
            ChangeKind::Added => self.added.push(change),
            ChangeKind::Changed => self.changed.push(change),
            ChangeKind::Deprecated => self.deprecated.push(change),
            ChangeKind::Removed => self.removed.push(change),
            ChangeKind::Fixed => self.fixed.push(change),
            ChangeKind::Security => self.security.push(change),
        }
    }
}

impl Display for Changes {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut first_printed = false;

        if !self.added.is_empty() {
            ensure_newline(f, &mut first_printed)?;
            writeln!(f, "### Added")?;
            print_changes(f, &self.added)?;
        }

        if !self.changed.is_empty() {
            ensure_newline(f, &mut first_printed)?;
            writeln!(f, "### Changed")?;
            print_changes(f, &self.changed)?;
        }

        if !self.deprecated.is_empty() {
            ensure_newline(f, &mut first_printed)?;
            writeln!(f, "### Deprecated")?;
            print_changes(f, &self.deprecated)?;
        }

        if !self.removed.is_empty() {
            ensure_newline(f, &mut first_printed)?;
            writeln!(f, "### Removed")?;
            print_changes(f, &self.removed)?;
        }

        if !self.fixed.is_empty() {
            ensure_newline(f, &mut first_printed)?;
            writeln!(f, "### Fixed")?;
            print_changes(f, &self.fixed)?;
        }

        if !self.security.is_empty() {
            ensure_newline(f, &mut first_printed)?;
            writeln!(f, "### Security")?;
            print_changes(f, &self.security)?;
        }

        Ok(())
    }
}

fn ensure_newline(f: &mut Formatter, first_printed: &mut bool) -> fmt::Result {
    if *first_printed {
        writeln!(f)?;
    } else {
        *first_printed = true;
    }

    Ok(())
}

fn print_changes(f: &mut Formatter, changes: &[String]) -> fmt::Result {
    changes.iter().try_for_each(|change| {
        let mut title = change
            .split('\n')
            .map(|line| format!("  {line}").trim_end().to_string())
            .collect::<Vec<String>>();
        title[0] = format!("- {}", substring(title[0].clone(), 1));
        writeln!(f, "{}", title.join("\n"))
    })
}
