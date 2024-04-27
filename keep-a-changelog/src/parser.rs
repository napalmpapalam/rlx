use eyre::{eyre, Context, Result};
use regex::Regex;
use semver::Version;

use crate::{
    changelog::ChangelogBuilder,
    release::{Release, ReleaseBuilder},
    token::{tokenize, Token, TokenKind},
    utils::{get_content, get_text_content},
    ChangeLogParseOptions, Changelog,
};

pub struct Parser {
    builder: ChangelogBuilder,
    tokens: Vec<Token>,
    opts: ChangeLogParseOptions,
}

impl Parser {
    pub fn new(markdown: String, opts: ChangeLogParseOptions) -> Result<Self> {
        let mut tokens = tokenize(markdown).wrap_err_with(|| "Failed to tokenize markdown")?;
        let mut builder = ChangelogBuilder::default();

        builder
            .flag(get_content(&mut tokens, vec![TokenKind::Flag], false)?)
            .title(get_content(&mut tokens, vec![TokenKind::H1], true)?)
            .description(get_text_content(&mut tokens)?)
            .url(opts.url.clone())
            .tag_prefix(opts.tag_prefix.clone());

        if let Some(head) = opts.head.clone() {
            builder.head(head);
        }

        Ok(Self {
            builder: builder.clone(),
            tokens,
            opts,
        })
    }

    pub fn parse(&mut self) -> Result<Changelog> {
        self.parse_releases()?
            .parse_links()?
            .parse_footer()?
            .build()
    }

    fn parse_releases(&mut self) -> Result<&mut Self> {
        let mut releases: Vec<Release> = vec![];
        let mut release = get_content(&mut self.tokens, vec![TokenKind::H2], false)?;
        let unreleased_regex = Regex::new(r"\[?([^\]]+)\]?\s*-\s*unreleased(\s+\[yanked\])?$")?;
        let release_regex =
            Regex::new(r"\[?([^\]]+)\]?\s*-\s*([\d]{4}-[\d]{1,2}-[\d]{1,2})(\s+\[yanked\])?$")?;

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

            release_builder.description(get_text_content(&mut self.tokens)?);

            let mut change_type = get_content(&mut self.tokens, vec![TokenKind::H3], false)?;

            while change_type.is_some() {
                let c_type = change_type.clone().unwrap().to_lowercase();

                let mut change = get_content(&mut self.tokens, vec![TokenKind::Li], false)?;

                while change.is_some() {
                    release_builder.add_change(c_type.clone(), change.clone().unwrap())?;
                    change = get_content(&mut self.tokens, vec![TokenKind::Li], false)?;
                }

                change_type = get_content(&mut self.tokens, vec![TokenKind::H3], false)?;
            }

            releases.push(release_builder.build()?);
            release = get_content(&mut self.tokens, vec![TokenKind::H2], false)?;
        }

        self.builder.releases(releases);

        Ok(self)
    }

    fn parse_links(&mut self) -> Result<&mut Self> {
        let release_link_regex = Regex::new(r"^\[.*\]\:\s*(http.*?)\/(?:-\/)?compare\/.*$")?;

        let mut links = vec![];

        while let Some(link) = get_content(&mut self.tokens, vec![TokenKind::Link], false)? {
            links.push(link.clone());

            if self.opts.url.is_some() {
                continue;
            }
            if let Some(captures) = release_link_regex.captures(&link) {
                self.builder.url(Some(captures[1].to_string()));
            }
        }

        self.builder.links(links)?;
        Ok(self)
    }

    fn parse_footer(&mut self) -> Result<&mut Self> {
        self.builder
            .footer(get_content(&mut self.tokens, vec![TokenKind::Hr], false)?);
        Ok(self)
    }

    fn build(&self) -> Result<Changelog> {
        if !self.tokens.is_empty() {
            return Err(eyre!("Unexpected tokens: {:?}", self.tokens));
        }

        self.builder
            .build()
            .wrap_err_with(|| "Failed to build Changelog")
    }
}
