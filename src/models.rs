use std::{
    convert::{TryFrom, TryInto},
    str::FromStr,
};

use anyhow::ensure;
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct CrateName(String);

impl TryFrom<String> for CrateName {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        ensure!(
            !value.is_empty()
                && ('a'..='z').contains(&value.chars().next().unwrap_or_default())
                && value
                    .chars()
                    .all(|c| matches!(c, '0'..='9' | 'a'..='z' | '-' | '_')),
            "invalid crate name"
        );

        Ok(Self(value))
    }
}

impl FromStr for CrateName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.to_owned().try_into()
    }
}

impl AsRef<str> for CrateName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
