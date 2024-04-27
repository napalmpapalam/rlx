use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use chrono::NaiveDate;
use derive_builder::Builder;
use derive_getters::Getters;
use eyre::{Context, OptionExt, Result};
use semver::Version;
use serde::{Serialize, Serializer};

use crate::{
    changes::{ChangeKind, Changes},
    link::Link,
    Changelog,
};

#[derive(Debug, Clone, Builder, Getters, Serialize, PartialEq, Eq)]
pub struct Release {
    #[serde(serialize_with = "version_serialize")]
    #[builder(setter(strip_option, into), default)]
    version: Option<Version>,
    #[builder(default = "false")]
    yanked: bool,
    #[builder(setter(into), default)]
    description: Option<String>,
    #[builder(setter(strip_option, into), default)]
    date: Option<NaiveDate>,
    #[builder(default)]
    changes: Changes,
}

impl ReleaseBuilder {
    pub fn add_change(&mut self, kind: String, change: String) -> Result<&mut Self> {
        let mut changes = self.changes.clone().unwrap_or_default();
        let kind = ChangeKind::from_str(kind.as_str())
            .wrap_err_with(|| format!("Failed to parse change kind: {kind}"))?;
        changes.add(kind, change);
        self.changes = Some(changes);
        Ok(self)
    }
}

impl Release {
    pub(crate) fn compare_link(&self, changelog: &Changelog) -> Result<Option<Link>> {
        let index = changelog
            .releases()
            .iter()
            .position(|release| release == self)
            .ok_or_eyre("Release not found")?;

        let mut offset = 1_usize;
        let mut previous = changelog.releases().get(index + offset);

        while let Some(prv) = previous {
            if prv.date().is_some() {
                break;
            }

            offset += 1_usize;
            previous = changelog.releases().get(index + offset);
        }

        if previous.is_none() && (self.date.is_none() || self.version.is_none()) {
            return Ok(None);
        }

        changelog.compare_link(self, previous)
    }
}

impl Ord for Release {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.date.cmp(&other.date)
    }
}

impl PartialOrd for Release {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Release {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let yanked = if self.yanked { " [YANKED]" } else { "" };

        if let Some(version) = self.version.clone() {
            let date = self
                .date
                .ok_or_eyre(format!("Missing date: {version}"))
                .map_err(|_| std::fmt::Error)?;
            let date = date.format("%Y-%m-%d").to_string();
            writeln!(f, "## [{version}] - {date}{yanked}")?;
        } else {
            writeln!(f, "## [Unreleased]{yanked}")?;
        }

        if let Some(description) = &self.description {
            writeln!(f, "{description}")?;
        }

        write!(f, "{}", self.changes)?;

        Ok(())
    }
}

fn version_serialize<S>(x: &Option<Version>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match x {
        Some(ref version) => s.serialize_str(&version.to_string()),
        None => s.serialize_none(),
    }
}
