//! All data structures related to the crate index.

use std::collections::{BTreeMap, BTreeSet};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use crate::api::models::{Dependency as ApiDependency, Kind as ApiKind, PublishRequest};
use crate::models::CrateName;

/// The configuration of the index. This structure is placed as a JSON file with the name
/// `config.json` at the root of the index repository. It tells cargo where to find the crate
/// repository's API and where to download the crate data.
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Download path for crate packages.
    pub dl: Url,
    /// Main API endpoint for the crate repository.
    pub api: Url,
}

/// A single release of a crate. It describes all basic information about a crate release and is
/// stored within the index.
#[derive(Serialize, Deserialize)]
pub struct Release {
    /// Name of the crate.
    pub name: CrateName,
    /// SemVer version of this release.
    pub vers: Version,
    /// List of dependencies that this crate uses.
    pub deps: Vec<Dependency>,
    /// SHA-256 checksum of the crate tarball.
    pub cksum: String,
    /// List of features that the crate supports.
    pub features: BTreeMap<String, BTreeSet<String>>,
    /// Whether this release is yanked (disabled for download).
    pub yanked: bool,
    /// Linking value that is passed to the compiler to link in external libraries.
    pub links: Option<String>,
}

impl From<(PublishRequest, &[u8])> for Release {
    fn from((p, d): (PublishRequest, &[u8])) -> Self {
        Self {
            name: p.name,
            vers: p.vers,
            deps: p.deps.into_iter().map(Into::into).collect(),
            cksum: hex::encode(Sha256::digest(d)),
            features: p.features,
            yanked: false,
            links: p.links,
        }
    }
}

/// A dependency describes the reference from a crate [`Release`] to another existing crate that it
/// uses.
#[derive(Serialize, Deserialize)]
pub struct Dependency {
    /// Name of this dependency. Not a [`CrateName`] as this crate could come from another registry
    /// with different naming restrictions.
    pub name: String,
    /// SemVer version requirement specification. For example `^0.6`.
    pub req: VersionReq,
    /// List of activated features on this crate as requested by the [`Release`].
    pub features: BTreeSet<String>,
    /// Whether this dependency is optional.
    pub optional: bool,
    /// Whether default features are enabled.
    pub default_features: bool,
    /// Target restrictions for this dependency. It is only needed if the requirement is fullfilled.
    pub target: Option<String>,
    /// One of the known kinds of dependencies.
    pub kind: Kind,
    /// The registry where this crate can be found. If missing, <https://crates.io> is used.
    pub registry: Option<Url>,
    /// Original name of the crate in case it was renamed by the [`Release`].
    pub package: Option<String>,
}

impl From<ApiDependency> for Dependency {
    fn from(d: ApiDependency) -> Self {
        Self {
            name: d.name,
            req: d.version_req,
            features: d.features,
            optional: d.optional,
            default_features: d.default_features,
            target: d.target,
            kind: d.kind.into(),
            registry: d.registry,
            package: d.explicit_name_in_toml,
        }
    }
}

/// Different kinds of dependencies. This means in what stage of the build process a dependency is
/// needed.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// Only during development like tests, benchmarks or examples.
    Dev,
    /// Only used in custom build scripts.
    Build,
    /// A normal dependency, directly used by the crate. Also available for development targets.
    Normal,
}

impl From<ApiKind> for Kind {
    fn from(k: ApiKind) -> Self {
        match k {
            ApiKind::Dev => Self::Dev,
            ApiKind::Build => Self::Build,
            ApiKind::Normal => Self::Normal,
        }
    }
}

#[cfg(test)]
mod tests {
    use maplit::{btreemap, btreeset};

    use super::*;

    #[test]
    fn serialize_config() {
        println!(
            "{}",
            serde_json::to_string_pretty(&Config {
                dl: "https://crates.io/api/v1/crates".parse().unwrap(),
                api: "https://crates.io".parse().unwrap(),
            })
            .unwrap()
        );
    }

    #[test]
    fn serialize_release() {
        println!(
            "{}",
            serde_json::to_string_pretty(&Release {
                name: "foo".parse().unwrap(),
                vers: "0.1.0".parse().unwrap(),
                deps: vec![Dependency {
                    name: "rand".to_owned(),
                    req: "^0.6".parse().unwrap(),
                    features: btreeset!["i128_support".to_owned()],
                    optional: false,
                    default_features: true,
                    target: None,
                    kind: Kind::Normal,
                    registry: None,
                    package: None,
                }],
                cksum: "d867001db0e2b6e0496f9fac96930e2d42233ecd3ca0413e0753d4c7695d289c"
                    .to_owned(),
                features: btreemap! {
                    "extras".to_owned() => btreeset!["rand/simd_support".to_owned()],
                },
                yanked: false,
                links: None,
            })
            .unwrap()
        );
    }
}
