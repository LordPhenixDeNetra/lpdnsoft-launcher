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
