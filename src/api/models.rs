use std::collections::{BTreeMap, BTreeSet};

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::models::CrateName;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ErrorResponse {
    pub errors: Vec<ErrorDetail>,
}

#[derive(Serialize, Deserialize)]
pub struct ErrorDetail {
    pub detail: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishRequest {
    pub name: CrateName,
    pub vers: Version,
    pub deps: Vec<Dependency>,
    pub features: BTreeMap<String, BTreeSet<String>>,
    pub authors: BTreeSet<String>,
    pub description: Option<String>,
    pub documentation: Option<Url>,
    pub homepage: Option<Url>,
    pub readme: Option<String>,
    pub readme_file: Option<String>,
    pub keywords: BTreeSet<String>,
    pub categories: BTreeSet<String>,
    pub license: Option<String>,
    pub license_file: Option<String>,
    pub repository: Option<Url>,
    pub badges: BTreeMap<String, BTreeMap<String, String>>,
    pub links: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version_req: VersionReq,
    pub features: BTreeSet<String>,
    pub optional: bool,
    pub default_features: bool,
    pub target: Option<String>,
    pub kind: Kind,
    pub registry: Option<Url>,
    pub explicit_name_in_toml: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Dev,
    Build,
    Normal,
}

#[derive(Serialize, Deserialize)]
pub struct PublishResponse {
    pub warnings: Warnings,
}

#[derive(Serialize, Deserialize)]
pub struct Warnings {
    pub invalid_categories: BTreeSet<String>,
    pub invalid_badges: BTreeSet<String>,
    pub other: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct YankResponse {
    pub ok: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UnyankResponse {
    pub ok: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ListOwnersResponse {
    pub users: Vec<User>,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub login: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AddOwnersRequest {
    pub users: BTreeSet<String>,
}

#[derive(Serialize, Deserialize)]
pub struct AddOwnersResponse {
    pub ok: bool,
    pub msg: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveOwnersRequest {
    pub users: BTreeSet<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveOwnersResponse {
    pub ok: bool,
    pub msg: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResponse {
    pub crates: Vec<Crate>,
    pub meta: Meta,
}

#[derive(Serialize, Deserialize)]
pub struct Crate {
    pub name: CrateName,
    pub max_version: Version,
    pub description: String,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    pub total: u64,
}

#[cfg(test)]
mod tests {
    use maplit::{btreemap, btreeset};

    use super::*;

    #[test]
    fn serialize_error_response() {
        println!(
            "{}",
            serde_json::to_string_pretty(&ErrorResponse {
                errors: vec![ErrorDetail {
                    detail: "error message text".to_owned()
                }]
            })
            .unwrap()
        );
    }

    #[test]
    fn serialize_publish_request() {
        println!(
            "{}",
            serde_json::to_string_pretty(&PublishRequest {
                name: "foo".parse().unwrap(),
                vers: "0.1.0".parse().unwrap(),
                deps: vec![Dependency {
                    name: "rand".to_owned(),
                    version_req: "^0.6".parse().unwrap(),
                    features: btreeset!["i128_support".to_owned()],
                    optional: false,
                    default_features: true,
                    target: None,
                    kind: Kind::Normal,
                    registry: None,
                    explicit_name_in_toml: None,
                }],
                features: btreemap! {
                    "extras".to_owned() => btreeset!["rand/simd_support".to_owned()],
                },
                authors: btreeset!["Alice <a@example.com>".to_owned()],
                description: None,
                documentation: None,
                homepage: None,
                readme: None,
                readme_file: None,
                keywords: btreeset![],
                categories: btreeset![],
                license: None,
                license_file: None,
                repository: None,
                badges: btreemap! {
                    "travis-ci".to_owned() => btreemap! {
                        "branch".to_owned() => "master".to_owned(),
                        "repository".to_owned() => "rust-lang/cargo".to_owned(),
                    },
                },
                links: None,
            })
            .unwrap()
        );
    }

    #[test]
    fn serialize_publish_response() {
        println!(
            "{}",
            serde_json::to_string_pretty(&PublishResponse {
                warnings: Warnings {
                    invalid_categories: btreeset![],
                    invalid_badges: btreeset![],
                    other: vec![],
                }
            })
            .unwrap()
        );
    }

    #[test]
    fn serialize_yank_response() {
        println!(
            "{}",
            serde_json::to_string_pretty(&YankResponse { ok: true }).unwrap()
        );
    }

    #[test]
    fn serialize_unyank_response() {
        println!(
            "{}",
            serde_json::to_string_pretty(&UnyankResponse { ok: true }).unwrap()
        );
    }

    #[test]
    fn serialize_list_owners_response() {
        println!(
            "{}",
            serde_json::to_string_pretty(&ListOwnersResponse {
                users: vec![User {
                    id: 70,
                    login: "github:rust-lang:core".to_owned(),
                    name: Some("Core".to_owned())
                }]
            })
            .unwrap()
        );
    }

    #[test]
    fn serialize_add_owners_request() {
        println!(
            "{}",
            serde_json::to_string_pretty(&AddOwnersRequest {
                users: btreeset!["login_name".to_owned()]
            })
            .unwrap()
        );
    }

    #[test]
    fn serialize_add_owners_response() {
        println!(
            "{}",
            serde_json::to_string_pretty(&AddOwnersResponse {
                ok: true,
                msg: "user ehuss has been invited to be an owner of crate cargo".to_owned()
            })
            .unwrap()
        );
    }

    #[test]
    fn serialize_remove_owners_request() {
        println!(
            "{}",
            serde_json::to_string_pretty(&RemoveOwnersRequest {
                users: btreeset!["login_name".to_owned()]
            })
            .unwrap()
        );
    }

    #[test]
    fn serialize_remove_owners_response() {
        println!(
            "{}",
            serde_json::to_string_pretty(&RemoveOwnersResponse {
                ok: true,
                msg: "user ehuss has been removed from the owner list of crate cargo".to_owned(),
            })
            .unwrap()
        );
    }

    #[test]
    fn serialize_search_response() {
        println!(
            "{}",
            serde_json::to_string_pretty(&SearchResponse {
                crates: vec![Crate {
                    name: "rand".parse().unwrap(),
                    max_version: "0.6.1".parse().unwrap(),
                    description: "Random number generators and other randomness functionality.\n"
                        .to_owned(),
                }],
                meta: Meta { total: 119 }
            })
            .unwrap()
        );
    }
}
