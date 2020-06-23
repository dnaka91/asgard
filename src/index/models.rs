use std::collections::{BTreeMap, BTreeSet};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::api::models::{Dependency as ApiDependency, Kind as ApiKind, PublishRequest};
use crate::models::CrateName;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub dl: Url,
    pub api: Url,
}

#[derive(Serialize, Deserialize)]
pub struct Release {
    pub name: CrateName,
    pub vers: Version,
    pub deps: Vec<Dependency>,
    pub cksum: String,
    pub features: BTreeMap<String, BTreeSet<String>>,
    pub yanked: bool,
    pub links: Option<String>,
}

impl From<PublishRequest> for Release {
    fn from(p: PublishRequest) -> Self {
        Self {
            name: p.name,
            vers: p.vers,
            deps: p.deps.into_iter().map(Into::into).collect(),
            cksum: String::from("TODO"),
            features: p.features,
            yanked: false,
            links: p.links,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub req: VersionReq,
    pub features: BTreeSet<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: Kind,
    pub registry: Option<Url>,
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Dev,
    Build,
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
