use eyre::{bail, Context, Result};
use regex::Regex;
use semver::Version;

use crate::{
    changelog::ChangelogBuilder,
    release::{Release, ReleaseBuilder},
    token::{tokenize, Token, TokenKind},
    ChangeLogParseOptions, Changelog,
};

pub struct Parser {
    builder: ChangelogBuilder,
    tokens: Vec<Token>,
    opts: ChangeLogParseOptions,
    idx: usize,
}

impl Parser {
    pub fn parse(markdown: String, opts: ChangeLogParseOptions) -> Result<Changelog> {
        let tokens = tokenize(markdown).wrap_err_with(|| "Failed to tokenize markdown")?;
        let builder = ChangelogBuilder::default();

        Self {
            builder,
            tokens,
            opts,
            idx: 0,
        }
        .parse_opts()?
        .parse_meta()?
        .parse_releases()?
        .parse_links()?
        .parse_footer()?
        .build()
    }

    fn parse_opts(&mut self) -> Result<&mut Self> {
        self.builder
            .url(self.opts.url.clone())
            .tag_prefix(self.opts.tag_prefix.clone());

        if let Some(head) = self.opts.head.clone() {
            self.builder.head(head);
        }

        Ok(self)
    }

    fn parse_meta(&mut self) -> Result<&mut Self> {
        let flag = self.get_content(vec![TokenKind::Flag], false)?;
        let title = self.get_content(vec![TokenKind::H1], true)?;
        let description = self.get_text_content()?;

        self.builder
            .flag(flag)
            .title(title)
            .description(description);

        Ok(self)
    }

    fn parse_releases(&mut self) -> Result<&mut Self> {
        let mut releases: Vec<Release> = vec![];
        let unreleased_regex = Regex::new(r"\[?([^\]]+)\]?\s*-\s*unreleased(\s+\[yanked\])?$")?;
        let release_regex =
            Regex::new(r"\[?([^\]]+)\]?\s*-\s*([\d]{4}-[\d]{1,2}-[\d]{1,2})(\s+\[yanked\])?$")?;

        while let Some(release) = self.get_content(vec![TokenKind::H2], false)? {
            let mut builder = ReleaseBuilder::default();
            let release = release.clone().to_lowercase();

            builder.yanked(release.contains("[yanked]"));

            if let Some(captures) = release_regex.captures(&release) {
                let version =
                    Version::parse(&captures[1]).wrap_err_with(|| "Failed to parse version")?;

                let date = chrono::NaiveDate::parse_from_str(&captures[2], "%Y-%m-%d")
                    .wrap_err_with(|| "Failed to parse date")?;

                builder.version(version).date(date);
            } else if release.contains("unreleased") {
                if let Some(captures) = unreleased_regex.captures(&release) {
                    builder.version(Version::parse(&captures[1])?);
                }
            } else {
                bail!("Failed to parse release: {:?}", release)
            }

            builder.description(self.get_text_content()?);

            while let Some(change_kind) = self.get_content(vec![TokenKind::H3], false)? {
                let change_kind = change_kind.to_lowercase();

                while let Some(change) = self.get_content(vec![TokenKind::Li], false)? {
                    builder.add_change(change_kind.clone(), change)?;
                }
            }

            releases.push(builder.build()?);
        }

        self.builder.releases(releases);

        Ok(self)
    }

    fn parse_links(&mut self) -> Result<&mut Self> {
        let release_link_regex = Regex::new(r"^\[.*\]\:\s*(http.*?)\/(?:-\/)?compare\/.*$")?;

        let mut links = vec![];

        while let Some(link) = self.get_content(vec![TokenKind::Link], false)? {
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
        let footer = self.get_content(vec![TokenKind::Hr], false)?;
        self.builder.footer(footer);
        Ok(self)
    }

    fn build(&self) -> Result<Changelog> {
        if self.idx != self.tokens.len() {
            bail!(
                "Unexpected tokens: {:?}, index: {}, tokens length: {}",
                self.tokens[self.idx..].to_vec(),
                self.idx,
                self.tokens.len(),
            );
        }

        self.builder
            .build()
            .wrap_err_with(|| "Failed to build Changelog")
    }

    fn get_content(&mut self, kinds: Vec<TokenKind>, required: bool) -> Result<Option<String>> {
        let token = self.tokens.get(self.idx);

        if token.is_none() {
            if required {
                bail!("Required token missing in line: {}", self.idx);
            }
            return Ok(None);
        }

        let token = token.unwrap();

        if !kinds.iter().any(|k| *k == token.kind) {
            if required {
                bail!("Required token kind missing in line: {}", self.idx);
            }
            return Ok(None);
        }

        self.idx += 1;
        Ok(Some(token.content.join("\n")))
    }

    fn get_text_content(&mut self) -> Result<Option<String>> {
        let mut lines: Vec<String> = vec![];
        let kinds = [TokenKind::P, TokenKind::Li];

        while let Some(token) = self.tokens.get(self.idx) {
            if !kinds.iter().any(|tt| *tt == token.kind) {
                break;
            }

            self.idx += 1;

            if token.kind == TokenKind::Li {
                lines.push(format!("- {}", token.content.join("\n")));
            } else {
                lines.push(token.content.join("\n"));
            }
        }

        if lines.is_empty() {
            return Ok(None);
        }

        Ok(Some(lines.join("\n")))
    }
}
