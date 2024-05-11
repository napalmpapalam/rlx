use crate::context::Context;
use eyre::Result;
use semver::Version;

pub(crate) trait VersionExt {
    fn validate(ctx: &Context, version: String) -> Result<bool>;
}

impl VersionExt for Version {
    fn validate(ctx: &Context, version: String) -> Result<bool> {
        let semver = version.parse::<Version>();

        match semver {
            Ok(_) => {
                ctx.success("Release version is compatible with semantic versioning");
                Ok(true)
            }
            Err(e) => {
                ctx.error(
                    format!("Release version is not compatible with semantic versioning: {e}")
                        .as_str(),
                );
                Ok(false)
            }
        }
    }
}
