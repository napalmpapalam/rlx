use crate::{context::Context, error::Result};

use keep_a_changelog::{Changelog, ChangelogParseOptions};

pub(crate) trait ChangelogExt {
    fn from_ctx(ctx: &Context) -> Result<Changelog>;
}

impl ChangelogExt for Changelog {
    fn from_ctx(ctx: &Context) -> Result<Self> {
        ctx.debug("Parsing changelog");

        let result = Changelog::parse_from_file(
            ctx.changelog_path(),
            Some(ChangelogParseOptions {
                url: Some(ctx.remote_url()?.to_owned()),
                tag_prefix: ctx.tag_prefix(),
                head: Some(ctx.head()),
            }),
        )?;

        ctx.debug("Successfully parsed changelog");

        Ok(result)
    }
}
