use std::collections::{BTreeMap, BTreeSet};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub dl: Url,
    pub api: Url,
}

#[derive(Serialize, Deserialize)]
pub struct Release {
    pub name: String,
    pub vers: Version,
    pub deps: Vec<Dependency>,
    pub cksnum: String,
    pub features: BTreeMap<String, BTreeSet<String>>,
    pub yanked: bool,
    pub links: Option<String>,
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Dev,
    Build,
    Normal,
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
                name: "foo".to_owned(),
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
                cksnum: "d867001db0e2b6e0496f9fac96930e2d42233ecd3ca0413e0753d4c7695d289c"
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
