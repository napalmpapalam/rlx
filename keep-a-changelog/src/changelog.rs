use std::{
    fmt::{self, Display},
    fs::File,
    io::Read,
    path::Path,
};

use derive_builder::Builder;
use derive_getters::Getters;
use eyre::{eyre, Context, OptionExt, Result};
use regex::Regex;
use semver::Version;

use crate::{
    consts::{CHANGELOG_DESCRIPTION, CHANGELOG_TITLE},
    link::Link,
    release::{Release, ReleaseBuilder},
    token::{tokenize, Token, TokenKind},
    utils::{get_content, get_git_compare_url, get_git_release_url, get_text_content},
};

#[derive(Debug, Clone, Builder, Getters)]
pub struct Changelog {
    #[builder(setter(into), default)]
    flag: Option<String>,
    #[builder(setter(into))]
    title: Option<String>,
    #[builder(setter(into))]
    description: Option<String>,
    #[builder(default = "self.default_head()")]
    head: String,
    #[builder(setter(into), default)]
    footer: Option<String>,
    #[builder(setter(into), default)]
    url: Option<String>,
    #[builder(setter(custom), default)]
    releases: Vec<Release>,
    #[builder(setter(custom), default)]
    links: Vec<Link>,
    #[builder(setter(into), default)]
    tag_prefix: Option<String>,
}

impl ChangelogBuilder {
    fn default_head(&self) -> String {
        "HEAD".into()
    }

    fn releases(&mut self, releases: Vec<Release>) -> &mut Self {
        self.releases = Some(releases);
        self.sort_releases()
    }

    fn sort_releases(&mut self) -> &mut Self {
        let mut releases = self.releases.clone().unwrap_or_default();

        let unreleased: Option<Release> = releases
            .iter()
            .position(|r| r.version().is_none() && r.date().is_none())
            .map(|idx| releases.remove(idx));

        releases.sort_by(|a, b| b.cmp(a));

        if let Some(unreleased) = unreleased {
            releases.insert(0, unreleased);
        }

        self.releases = Some(releases);
        self
    }

    fn links(&mut self, links: Vec<String>) -> Result<&mut Self> {
        let links = links
            .iter()
            .map(|link| Link::parse(link.clone()))
            .collect::<Result<Vec<Link>>>()
            .wrap_err_with(|| "Failed to parse links")?;
        self.links = Some(links);
        Ok(self)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ChangeLogParseOptions {
    pub url: Option<String>,
    pub tag_prefix: Option<String>,
}

impl Changelog {
    pub fn parse(path: &str, opts: ChangeLogParseOptions) -> Result<Self> {
        let path = Path::new(path);
        let mut markdown = String::new();
        File::open(path)?
            .read_to_string(&mut markdown)
            .wrap_err_with(|| "Failed to read CHANGELOG.md")?;
        let tokens = tokenize(markdown).wrap_err_with(|| "Failed to tokenize markdown")?;
        process_tokens(tokens, opts)
    }

    pub fn find_release(&self, version: String) -> Result<Option<&Release>> {
        let version = Version::parse(&version).wrap_err_with(|| {
            format!("Failed to parse version: {version} during finding release")
        })?;

        Ok(self
            .releases()
            .iter()
            .find(|r| r.version() == &Some(version.clone())))
    }

    pub fn sort_releases(&mut self) -> &mut Self {
        let unreleased: Option<Release> = self
            .releases
            .iter()
            .position(|r| r.version().is_none() && r.date().is_none())
            .map(|idx| self.releases.remove(idx));

        self.releases.sort_by(|a, b| b.cmp(a));

        if let Some(unreleased) = unreleased {
            self.releases.insert(0, unreleased);
        }

        self
    }

    pub fn add_release(&mut self, release: Release) -> &mut Self {
        self.releases.insert(0, release);
        self.sort_releases()
    }

    pub(crate) fn compare_link(
        &self,
        current: &Release,
        previous: Option<&Release>,
    ) -> Result<Option<Link>> {
        let repo_url = self.url().clone().ok_or_eyre("Missing repo URL")?;

        if previous.is_none() {
            let version = current
                .version()
                .clone()
                .ok_or_eyre("Missing version for current release")?
                .to_string();
            return Ok(Some(Link {
                anchor: version.clone(),
                url: get_git_release_url(repo_url, self.tag_name(version)),
            }));
        }

        let previous = previous.unwrap();

        if current.date().is_none() || current.version().is_none() {
            let version = previous
                .version()
                .clone()
                .ok_or_eyre("Missing version for previous release")?
                .to_string();
            return Ok(Some(Link {
                anchor: "Unreleased".into(),
                url: get_git_compare_url(repo_url, self.tag_name(version), self.head().clone()),
            }));
        }

        let current_version = current
            .version()
            .clone()
            .ok_or_eyre("Missing version for current release")?
            .to_string();
        let previous_version = previous
            .version()
            .clone()
            .ok_or_eyre("Missing version for previous release")?
            .to_string();

        Ok(Some(Link {
            anchor: current_version.clone(),
            url: get_git_compare_url(
                repo_url,
                self.tag_name(previous_version),
                self.tag_name(current_version),
            ),
        }))
    }

    fn tag_name(&self, version: String) -> String {
        if let Some(tag_prefix) = self.tag_prefix() {
            return format!("{}{}", tag_prefix, version);
        }
        version.to_string()
    }
}

fn process_tokens(tokens: Vec<Token>, opts: ChangeLogParseOptions) -> Result<Changelog> {
    let release_link_regex = Regex::new(r"^\[.*\]\:\s*(http.*?)\/(?:-\/)?compare\/.*$")?;
    let unreleased_regex = Regex::new(r"\[?([^\]]+)\]?\s*-\s*unreleased(\s+\[yanked\])?$")?;
    let release_regex =
        Regex::new(r"\[?([^\]]+)\]?\s*-\s*([\d]{4}-[\d]{1,2}-[\d]{1,2})(\s+\[yanked\])?$")?;

    let mut tokens = tokens;
    let mut builder = ChangelogBuilder::default();

    builder
        .flag(get_content(&mut tokens, vec![TokenKind::Flag], false)?)
        .title(get_content(&mut tokens, vec![TokenKind::H1], true)?)
        .description(get_text_content(&mut tokens)?)
        .url(opts.url.clone())
        .tag_prefix(opts.tag_prefix.clone());

    let mut releases: Vec<Release> = vec![];
    let mut release = get_content(&mut tokens, vec![TokenKind::H2], false)?;

    while release.is_some() {
        let rel = release.clone().unwrap().to_lowercase();
        let captures = release_regex.captures(&rel);
        let mut release_builder = ReleaseBuilder::default();

        if captures.is_some() {
            let captures = captures.unwrap();
            let version = captures.get(1).unwrap().clone().as_str();
            let version = Version::parse(version)
                .wrap_err_with(|| format!("Failed to parse version: {version}"))?;
            let date = captures.get(2).unwrap().clone().as_str();
            let date = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
                .wrap_err_with(|| format!("Failed to parse date: {date}"))?;
            let yanked = captures.get(3).clone().is_some();

            release_builder.version(version).date(date).yanked(yanked);
        } else if rel.contains("unreleased") {
            release_builder.yanked(rel.contains("[yanked]"));
            let captures = unreleased_regex.captures(&rel);

            if let Some(captures) = captures {
                let version = Version::parse(captures.get(1).unwrap().as_str())?;
                release_builder.version(version);
            }
        } else {
            return Err(eyre!("Failed to parse release: {:?}", rel));
        }

        release_builder.description(get_text_content(&mut tokens)?);

        let mut change_type = get_content(&mut tokens, vec![TokenKind::H3], false)?;

        while change_type.is_some() {
            let c_type = change_type.clone().unwrap().to_lowercase();

            let mut change = get_content(&mut tokens, vec![TokenKind::Li], false)?;

            while change.is_some() {
                release_builder.add_change(c_type.clone(), change.clone().unwrap())?;
                change = get_content(&mut tokens, vec![TokenKind::Li], false)?;
            }

            change_type = get_content(&mut tokens, vec![TokenKind::H3], false)?;
        }

        releases.push(release_builder.build()?);
        release = get_content(&mut tokens, vec![TokenKind::H2], false)?;
    }

    builder.releases(releases);

    let mut links = vec![];

    while let Some(link) = get_content(&mut tokens, vec![TokenKind::Link], false)? {
        links.push(link.clone());

        if opts.url.is_some() {
            continue;
        }
        if let Some(captures) = release_link_regex.captures(&link) {
            builder.url(Some(captures[1].to_string()));
        }
    }

    builder.links(links)?;
    builder.footer(get_content(&mut tokens, vec![TokenKind::Hr], false)?);

    if !tokens.is_empty() {
        return Err(eyre!("Unexpected tokens: {:?}", tokens));
    }

    builder
        .build()
        .wrap_err_with(|| "Failed to build Changelog")
}

impl Display for Changelog {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(flag) = self.flag.clone() {
            writeln!(f, "<!-- ${flag} -->")?;
        }

        let title = self.title.clone().unwrap_or(CHANGELOG_TITLE.into());
        writeln!(f, "# {title}",)?;

        if let Some(description) = self.description.clone() {
            writeln!(f, "{}", description.trim().to_owned())?;
        } else {
            write!(f, "{CHANGELOG_DESCRIPTION}")?;
        }

        let mut compare_links: Vec<Option<Link>> = vec![];

        for release in self.releases.iter() {
            compare_links.push(
                release
                    .compare_link(self)
                    .wrap_err_with(|| "Failed to get compare link")
                    .map_err(|_| std::fmt::Error)?,
            );
            write!(f, "\n{release}")?;
        }

        writeln!(f)?;

        let tag_regex = Regex::new(r"\d+\.\d+\.\d+((-rc|-x)\.\d+)?").unwrap();

        let non_compare_links: Vec<&Link> = self
            .links
            .iter()
            .filter(|link| {
                !tag_regex.is_match(link.anchor()) && !link.anchor().contains("Unreleased")
            })
            .collect();

        non_compare_links
            .iter()
            .try_for_each(|link| write!(f, "\n{link}"))?;

        if !non_compare_links.is_empty() {
            writeln!(f)?;
        }

        compare_links
            .into_iter()
            .flatten()
            .try_for_each(|link| write!(f, "\n{link}"))?;

        if let Some(footer) = self.footer.clone() {
            write!(f, "---\n{footer}\n")?;
        }

        writeln!(f)?;

        Ok(())
    }
}
