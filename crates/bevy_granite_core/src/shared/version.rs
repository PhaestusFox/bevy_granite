use bevy_granite_logging::{log, LogCategory, LogLevel, LogType};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, ops::Deref, str::FromStr};

#[derive(Deserialize, Debug)]
struct FileVersionConfig {
    scene_format: SceneFormatConfig,
}

#[derive(Deserialize, Debug)]
struct SceneFormatConfig {
    current_version: Version,
    minimum_supported_version: Version,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Version {
    V0_1_4,
    V0_1_5,
}

impl Version {
    pub const CURRENT_VERSION: Version = Version::V0_1_4;
    pub const MINIMUM_SUPPORTED_VERSION: Version = Version::V0_1_4;
    pub const PRE_RELEASE_VERSION: Version = Version::V0_1_5;
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Version::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Version {
    pub fn major(&self) -> u32 {
        match self {
            Version::V0_1_4 => 0,
            Version::V0_1_5 => 0,
        }
    }

    pub fn minor(&self) -> u32 {
        match self {
            Version::V0_1_4 => 1,
            Version::V0_1_5 => 1,
        }
    }

    pub fn patch(&self) -> u32 {
        match self {
            Version::V0_1_4 => 4,
            Version::V0_1_5 => 5,
        }
    }

    pub fn suffix(&self) -> Option<&String> {
        None
    }

    pub fn is_pre_release(&self) -> bool {
        matches!(self, Version::V0_1_5)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Version::V0_1_4 => "0.1.4",
            Version::V0_1_5 => "0.1.5",
        }
    }
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0.1.4" => Ok(Version::V0_1_4),
            "0.1.5" => Ok(Version::V0_1_5),
            _ => Err(VersionError::InvalidVersion(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum VersionError {
    InvalidVersion(String),
}

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionError::InvalidVersion(v) => write!(f, "Invalid version string: {v}"),
        }
    }
}

impl std::error::Error for VersionError {}

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major.minor.patch first
        match (
            self.major().cmp(&other.major()),
            self.minor().cmp(&other.minor()),
            self.patch().cmp(&other.patch()),
        ) {
            (Ordering::Equal, Ordering::Equal, Ordering::Equal) => {
                // Core versions are equal, now compare pre-release
                match (&self.suffix(), &other.suffix()) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Greater, // Stable > pre-release
                    (Some(_), None) => Ordering::Less,    // Pre-release < stable
                    (Some(a), Some(b)) => {
                        // Both are pre-releases, compare lexicographically
                        // This is a simplified comparison
                        a.cmp(b)
                    }
                }
            }
            (Ordering::Equal, Ordering::Equal, patch_cmp) => patch_cmp,
            (Ordering::Equal, minor_cmp, _) => minor_cmp,
            (major_cmp, _, _) => major_cmp,
        }
    }
}

/// Check if the given version is compatible with the current format
pub fn is_scene_version_compatible(version: Version) -> bool {
    let current_version = Version::CURRENT_VERSION;
    let min_version = Version::MINIMUM_SUPPORTED_VERSION;

    // Check if version matches current exactly
    if version == current_version {
        log!(
            LogType::Game,
            LogLevel::Info,
            LogCategory::System,
            "Version '{}' matches current version exactly",
            version
        );
        return true;
    }

    // Check if version is at least the minimum supported
    if version >= min_version {
        if version < current_version {
            let version_type = if version.is_pre_release() {
                "pre-release"
            } else {
                "stable"
            };
            log!(
                LogType::Game,
                LogLevel::Info,
                LogCategory::System,
                "Loading older compatible {} version '{}' (current: '{}', min supported: '{}')",
                version_type,
                version,
                current_version,
                min_version
            );
        } else {
            // version > current_version
            let version_type = if version.is_pre_release() {
                "pre-release"
            } else {
                "stable"
            };
            log!(
                LogType::Game,
                LogLevel::Warning,
                LogCategory::System,
                "Loading newer {} version '{}' than current '{}' - this may cause issues",
                version_type,
                version.to_string(),
                current_version.to_string()
            );
        }
        return true;
    }

    // Version is below minimum supported
    let version_type = if version.is_pre_release() {
        "pre-release"
    } else {
        "stable"
    };
    log!(
        LogType::Game,
        LogLevel::Error,
        LogCategory::System,
        "Version '{}' ({}) is below minimum supported version '{}'. Current version is '{}'.",
        version,
        version_type,
        min_version,
        current_version
    );
    false
}

// impl std::fmt::Display for Version {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
//         if let Some(ref pre_release) = self.pre_release {
//             write!(f, "-{}", pre_release)?;
//         }
//         if let Some(ref build) = self.build {
//             write!(f, "+{}", build)?;
//         }
//         Ok(())
//     }
// }
