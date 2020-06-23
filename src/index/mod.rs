use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, ErrorKind};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use git2::{ErrorCode, Repository};

use self::models::Release;
use crate::api::models::PublishRequest;
use crate::models::CrateName;

pub mod models;

pub trait Service {
    fn add_crate(&self, req: PublishRequest) -> Result<()>;
}

pub struct ServiceImpl {
    repo: Repository,
}

impl ServiceImpl {
    fn read_latest_release(&self, crate_path: &Path) -> Result<Option<Release>> {
        let f = match File::open(crate_path) {
            Ok(f) => f,
            Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
            Err(e) => bail!(e),
        };

        let f = BufReader::new(f);

        Ok(match f.lines().last() {
            Some(line) => serde_json::from_str(&line?)?,
            None => None,
        })
    }
}

impl Service for ServiceImpl {
    fn add_crate(&self, req: PublishRequest) -> Result<()> {
        let path = crate_path(&req.name);
        let repo_path = self
            .repo
            .path()
            .parent()
            .unwrap_or_else(|| self.repo.path());
        let repo_path = repo_path.join(&path);

        if let Some(latest) = self.read_latest_release(&repo_path)? {
            if latest.vers <= req.vers {
                bail!("Only newer version allowed")
            }
        }

        fs::create_dir_all(&repo_path.parent().context("No parent file")?)?;

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&repo_path)?;

        let release = Release::from(req);
        writeln!(&mut file, "{}", serde_json::to_string(&release)?)?;

        self.repo.index()?.add_path(&path)?;
        let tree = self.repo.index()?.write_tree()?;
        let tree = self.repo.find_tree(tree)?;
        let signature = self.repo.signature()?;
        let head = self.repo.head()?.peel_to_commit()?;

        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("Publish crate \"{}@{}\"", release.name, release.vers),
            &tree,
            &[&head],
        )?;

        self.repo.checkout_head(None)?;

        Ok(())
    }
}

/// Create a new index service.
pub fn new(location: &Path) -> Result<impl Service> {
    let repo = match Repository::open(location) {
        Ok(r) => r,
        Err(e) => {
            if e.code() == ErrorCode::NotFound {
                Repository::init(location)?
            } else {
                bail!(e);
            }
        }
    };

    Ok(ServiceImpl { repo })
}

/// Crate paths are created according to the [Index Format](https://doc.rust-lang.org/cargo/reference/registries.html#index-format).
///
/// The rules are as follows:
/// - Packages with 1 character names are placed in a directory named `1`.
/// - Packages with 2 character names are placed in a directory named `2`.
/// - Packages with 3 character names are placed in the directory `3/{first-character}` where
///   `{first-character}` is the first character of the package name.
/// - All other packages are stored in directories named `{first-two}/{second-two}` where the top
///   directory is the first two characters of the package name, and the next subdirectory is the
///   third and fourth characters of the package name. For example, `cargo` would be stored in a
///   file named ca/rg/cargo.
fn crate_path(name: &CrateName) -> PathBuf {
    let name = name.as_ref();
    let path = match name.len() {
        1 => PathBuf::from("1"),
        2 => PathBuf::from("2"),
        3 => PathBuf::from("3").join(&name[..1]),
        _ => PathBuf::from(&name[..2]).join(&name[2..4]),
    };

    path.join(name)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::Path;
    use std::process::Command;

    use tempfile::TempDir;

    use super::*;

    fn create_repo() -> TempDir {
        let dir = tempfile::tempdir().unwrap();

        assert!(Command::new("git")
            .args(&["init"])
            .current_dir(dir.path())
            .status()
            .unwrap()
            .success());

        std::fs::write(dir.path().join("README.md"), &[]).unwrap();

        assert!(Command::new("git")
            .args(&["add", "."])
            .current_dir(dir.path())
            .status()
            .unwrap()
            .success());

        assert!(Command::new("git")
            .args(&["commit", "-m", "Initial commit"])
            .current_dir(dir.path())
            .status()
            .unwrap()
            .success());

        dir
    }

    #[test]
    fn service_add_crate() {
        let dir = create_repo();
        let service = new(dir.path()).unwrap();

        service
            .add_crate(PublishRequest {
                name: "test".parse().unwrap(),
                vers: "1.0.0".parse().unwrap(),
                deps: Vec::new(),
                features: BTreeMap::new(),
                authors: BTreeSet::new(),
                description: None,
                documentation: None,
                homepage: None,
                readme: None,
                readme_file: None,
                keywords: BTreeSet::new(),
                categories: BTreeSet::new(),
                license: Some("MIT".to_owned()),
                license_file: None,
                repository: None,
                badges: BTreeMap::new(),
                links: None,
            })
            .unwrap();
    }

    #[test]
    fn test_crate_path() {
        let table = &[
            ("1/a", "a"),
            ("2/ab", "ab"),
            ("3/a/abc", "abc"),
            ("ab/cd/abcd", "abcd"),
        ];

        for (expect, input) in table.iter() {
            assert_eq!(
                Path::new(expect),
                crate_path(&input.parse().unwrap()).as_path()
            );
        }
    }
}
