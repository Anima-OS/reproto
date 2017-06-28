mod file_index;
mod git_index;

use git;
use objects::Objects;
pub use reproto_core::{RpPackage, Version, VersionReq};
use self::file_index::*;
use self::git_index::*;
use sha256::Checksum;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use url::Url;

/// Configuration file for objects backends.
pub struct IndexConfig {
    /// Root path when checking out local repositories.
    pub repos: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Deployment {
    pub version: Version,
    pub object: Checksum,
}

impl Deployment {
    pub fn new(version: Version, object: Checksum) -> Deployment {
        Deployment {
            version: version,
            object: object,
        }
    }
}

use errors::*;

pub trait Index {
    fn resolve(&self,
               package: &RpPackage,
               version_req: Option<&VersionReq>)
               -> Result<Vec<Deployment>>;

    fn put_version(&self,
                   checksum: &Checksum,
                   package: &RpPackage,
                   version: &Version)
                   -> Result<()>;

    fn get_deployments(&self, package: &RpPackage, version: &Version) -> Result<Vec<Deployment>>;

    /// Get an objects URL as configured in the index.
    ///
    /// If relative, will cause objects to be loaded from the same repository as the index.
    fn objects_url(&self) -> Result<String>;

    /// Load objects relative to the index repository.
    fn objects_from_index(&self, relative_path: &Path) -> Result<Box<Objects>>;

    /// Update local caches related to the index.
    fn update(&self) -> Result<()> {
        Ok(())
    }
}

pub fn index_from_file(url: &Url) -> Result<Box<Index>> {
    let path = Path::new(url.path());

    if !path.is_dir() {
        return Err(format!("no such directory: {}", path.display()).into());
    }

    Ok(Box::new(file_index::FileIndex::new(&path)))
}

pub fn index_from_git<'a, I>(config: IndexConfig, scheme: I, url: &'a Url) -> Result<Box<Index>>
    where I: IntoIterator<Item = &'a str>
{
    let mut scheme = scheme.into_iter();

    let sub_scheme = scheme.next()
        .ok_or_else(|| format!("invalid scheme ({}), expected git+scheme", url.scheme()))?;

    let repos = config.repos.ok_or_else(|| "repos: not specified")?;

    let git_repo = Rc::new(git::setup_git_repo(&repos, sub_scheme, url)?);

    let file_objects = FileIndex::new(git_repo.path());
    let index = GitIndex::new(git_repo, file_objects);

    Ok(Box::new(index))
}

pub fn index_from_url(config: IndexConfig, url: &Url) -> Result<Box<Index>> {
    let mut scheme = url.scheme().split("+");
    let first = scheme.next().ok_or_else(|| format!("invalid scheme: {}", url))?;

    match first {
        "file" => index_from_file(url),
        "git" => index_from_git(config, scheme, url),
        scheme => Err(format!("unsupported scheme ({}): {}", scheme, url).into()),
    }
}