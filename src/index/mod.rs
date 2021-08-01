use std::{
    fs::{self, File, OpenOptions},
    io::{prelude::*, BufReader, BufWriter, ErrorKind},
    path::{Path, PathBuf},
};

use anyhow::{bail, ensure, Context, Result};
use git2::{build::CheckoutBuilder, ErrorCode, Repository, RepositoryInitOptions};
use parking_lot::Mutex;
use semver::Version;

use self::models::Release;
use crate::{api::models::PublishRequest, models::CrateName, settings};

pub mod models;

/// The index service that handles all functionality of the crate index. This index holds metadata
/// information about all crates like existing versions, dependencies and so on.
pub trait Service: Send + Sync + 'static {
    /// Add a new crate or version to the index. If previous versions exist, then the new version
    /// must have a higher semver version that any other version.
    fn add_crate(&self, req: PublishRequest, data: &[u8]) -> Result<()>;
    /// Yank or unyank a single version of an existing crate. This means that the version will not
    /// be available for download anymore (or be available again).
    fn yank(&self, name: CrateName, version: Version, yank: bool) -> Result<()>;
}

/// Main implementation of the index [`Service`].
pub struct ServiceImpl {
    repo: Mutex<Repository>,
}

impl ServiceImpl {
    /// Load all information about a single crate and get the latest release.
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

    /// Get the base path of the currently open repository.
    fn repo_path(&self) -> PathBuf {
        let repo = self.repo.lock();
        repo.path()
            .parent()
            .unwrap_or_else(|| repo.path())
            .to_owned()
    }

    /// Add and commit a single file to the index.
    fn commit_file(&self, path: &Path, message: &str) -> Result<()> {
        let repo = self.repo.lock();
        commit_file(&repo, path, message)
    }
}

impl Service for ServiceImpl {
    fn add_crate(&self, req: PublishRequest, data: &[u8]) -> Result<()> {
        let path = crate_path(&req.name);
        let repo_path = self.repo_path().join(&path);

        if let Some(latest) = self.read_latest_release(&repo_path)? {
            ensure!(latest.vers < req.vers, "only newer version allowed");
        }

        fs::create_dir_all(&repo_path.parent().context("no parent file")?)?;

        let mut file = BufWriter::new(
            OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open(&repo_path)?,
        );

        let release = Release::from((req, data));
        writeln!(&mut file, "{}", serde_json::to_string(&release)?)?;

        file.flush()?;

        self.commit_file(
            &path,
            &format!("Publish crate \"{}@{}\"", release.name, release.vers),
        )?;

        Ok(())
    }

    fn yank(&self, name: CrateName, version: Version, yank: bool) -> Result<()> {
        let path = crate_path(&name);
        let repo_path = self.repo_path().join(&path);

        let f = BufReader::new(File::open(&repo_path)?);

        let mut releases = f
            .lines()
            .map(|l| {
                l.map_err(Into::into)
                    .and_then(|l| serde_json::from_str(&l).map_err(Into::into))
            })
            .collect::<Result<Vec<Release>>>()?;

        let mut rel = releases
            .iter_mut()
            .find(|r| r.vers == version)
            .context("version doesn't exist")?;

        rel.yanked = yank;

        let mut f = BufWriter::new(File::create(repo_path)?);
        for rel in releases {
            writeln!(&mut f, "{}", serde_json::to_string(&rel)?)?;
        }

        f.flush()?;

        self.commit_file(
            &path,
            &format!(
                "{} crate \"{}@{}\"",
                if yank { "Yank" } else { "Unyank" },
                name,
                version
            ),
        )?;

        Ok(())
    }
}

/// Create a new index service.
pub fn new(settings: &settings::Index) -> Result<impl Service> {
    let repo = match Repository::open(&settings.location) {
        Ok(r) => r,
        Err(e) => {
            if e.code() == ErrorCode::NotFound {
                let mut options = RepositoryInitOptions::new();
                options.initial_head("master");
                options.description("Asgard crate index");

                let repo = Repository::init_opts(&settings.location, &options)?;
                create_initial_commit(&repo)?;

                repo
            } else {
                bail!(e);
            }
        }
    };

    update_config(&repo, settings)?;

    Ok(ServiceImpl {
        repo: Mutex::new(repo),
    })
}

fn update_config(repo: &Repository, settings: &settings::Index) -> Result<()> {
    let config_path = settings.location.join("config.json");

    let current_config = match File::open(&config_path) {
        Ok(file) => Some(serde_json::from_reader::<_, settings::IndexConfig>(file)?),
        Err(e) if e.kind() == ErrorKind::NotFound => None,
        Err(e) => bail!(e),
    };

    if Some(&settings.config) != current_config.as_ref() {
        let file = File::create(&config_path)?;
        serde_json::to_writer_pretty(file, &settings.config)?;

        commit_file(repo, "config.json", "Update config")?;
    }

    Ok(())
}

fn create_initial_commit(repo: &Repository) -> Result<()> {
    const README: &str = include_str!("README.md");

    let path = repo.path().parent().unwrap().join("README.md");
    fs::write(path, README)?;

    let sig = repo.signature()?;
    let tree = {
        let mut index = repo.index()?;
        index.add_path(Path::new("README.md"))?;

        let oid = index.write_tree()?;
        repo.find_tree(oid)?
    };

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;
    repo.checkout_head(Some(CheckoutBuilder::new().force()))?;

    Ok(())
}

fn commit_file(repo: &Repository, path: impl AsRef<Path>, message: &str) -> Result<()> {
    let sig = repo.signature()?;
    let tree = {
        let mut index = repo.index()?;
        index.add_path(path.as_ref())?;

        let oid = index.write_tree()?;
        repo.find_tree(oid)?
    };
    let parent = repo.head()?.peel_to_commit()?;

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent])?;

    repo.checkout_head(None)?;

    Ok(())
}

/// Crate paths are created according to the
/// [Index Format](https://doc.rust-lang.org/cargo/reference/registries.html#index-format).
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
    use std::{path::Path, process::Command};

    use tempfile::TempDir;

    use super::*;

    fn run_git(dir: &TempDir, args: &[&str]) {
        assert!(Command::new("git")
            .current_dir(dir.path())
            .args(args)
            .status()
            .unwrap()
            .success());
    }

    fn create_repo() -> TempDir {
        let dir = tempfile::tempdir().unwrap();

        run_git(&dir, &["init", "-q"]);
        run_git(&dir, &["config", "user.email", "test@test.com"]);
        run_git(&dir, &["config", "user.name", "Test"]);

        std::fs::write(dir.path().join("README.md"), &[]).unwrap();

        run_git(&dir, &["add", "."]);
        run_git(&dir, &["commit", "-q", "-m", "Initial commit"]);

        dir
    }

    #[test]
    fn service_roundtrip() {
        let dir = create_repo();
        let settings = settings::Index {
            location: dir.path().to_owned(),
            config: settings::IndexConfig {
                dl: "http://localhost:8080/api/v1/crates".parse().unwrap(),
                api: "http://localhost:8080".parse().unwrap(),
            },
        };
        let service = new(&settings).unwrap();

        service
            .add_crate(
                PublishRequest::new("test".parse().unwrap(), "1.0.0".parse().unwrap()),
                &[],
            )
            .unwrap();

        service
            .add_crate(
                PublishRequest::new("test".parse().unwrap(), "1.1.0".parse().unwrap()),
                &[1, 2, 3],
            )
            .unwrap();

        service
            .yank("test".parse().unwrap(), "1.0.0".parse().unwrap(), true)
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
