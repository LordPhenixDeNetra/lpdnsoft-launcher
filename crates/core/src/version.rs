//! Versions publiées : `major.minor.patch`, avec un `v` facultatif.

use std::fmt;
use std::str::FromStr;

/// Version publiée d'une application.
///
/// Les préversions ne sont pas des versions installables : `1.0.0-rc1` est refusé, comme
/// dans la vérification de mise à jour d'aNkou.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("version illisible : « {0} »")]
pub struct VersionError(pub String);

impl Version {
    pub const fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        let body = text.strip_prefix('v').unwrap_or(text);
        let invalid = || VersionError(text.to_owned());
        let mut parts = body.split('.');
        let number = |parts: &mut std::str::Split<'_, char>| -> Result<u64, VersionError> {
            let part = parts.next().ok_or_else(invalid)?;
            if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
                return Err(invalid());
            }
            part.parse().map_err(|_| invalid())
        };
        let major = number(&mut parts)?;
        let minor = number(&mut parts)?;
        let patch = number(&mut parts)?;
        if parts.next().is_some() {
            return Err(invalid());
        }
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(text: &str) -> Version {
        text.parse().unwrap()
    }

    #[test]
    fn parses_plain_and_tagged_versions() {
        assert_eq!(version("1.0.1"), Version::new(1, 0, 1));
        assert_eq!(version("v1.0.1"), Version::new(1, 0, 1));
        assert_eq!(version(" 12.340.5 "), Version::new(12, 340, 5));
    }

    #[test]
    fn rejects_what_is_not_a_release() {
        for text in [
            "",
            "1",
            "1.2",
            "1.2.3.4",
            "1.2.3-rc1",
            "nightly",
            "v",
            "1..2",
            "-1.0.0",
        ] {
            assert!(
                text.parse::<Version>().is_err(),
                "{text} aurait dû être refusé"
            );
        }
    }

    #[test]
    fn orders_by_number_not_by_text() {
        assert!(version("1.10.0") > version("1.9.0"));
        assert!(version("2.0.0") > version("1.99.99"));
        assert_eq!(version("1.0.0"), version("v1.0.0"));
    }

    #[test]
    fn displays_canonically() {
        assert_eq!(version("v01.2.3").to_string(), "1.2.3");
    }
}
