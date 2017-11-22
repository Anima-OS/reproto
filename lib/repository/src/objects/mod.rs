mod file_objects;
mod git_objects;
mod http_objects;
mod cached_objects;

pub use self::cached_objects::CachedObjects;
pub use self::file_objects::FileObjects;
pub use self::git_objects::GitObjects;
pub use self::http_objects::HttpObjects;
use checksum::Checksum;
use core::Object;
use errors::*;
use git;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;
use tokio_core::reactor::Core;
use update::Update;
use url::Url;

/// Configuration file for objects backends.
pub struct ObjectsConfig {
    /// Root path when checking out local repositories.
    pub repo_dir: PathBuf,
    pub cache_dir: Option<PathBuf>,
    pub missing_cache_time: Option<Duration>,
}

pub trait Objects {
    /// Put the given object into the database.
    /// This will cause the object denoted by the given checksum to be uploaded to the objects
    /// store.
    fn put_object(&mut self, checksum: &Checksum, source: &mut Read, force: bool) -> Result<()>;

    /// Get a path to the object with the given checksum.
    /// This might cause the object to be downloaded if it's not already present in the local
    /// filesystem.
    fn get_object(&mut self, checksum: &Checksum) -> Result<Option<Box<Object>>>;

    /// Update local caches related to the object store.
    fn update(&self) -> Result<Vec<Update>> {
        Ok(vec![])
    }
}

pub struct NoObjects;

impl Objects for NoObjects {
    fn put_object(&mut self, _: &Checksum, _: &mut Read, _: bool) -> Result<()> {
        Err(ErrorKind::EmptyObjects.into())
    }

    fn get_object(&mut self, _: &Checksum) -> Result<Option<Box<Object>>> {
        Err(ErrorKind::EmptyObjects.into())
    }
}

/// Load objects from a path.
pub fn objects_from_path<P: AsRef<Path>>(path: P) -> Result<FileObjects> {
    let path = path.as_ref();

    if !path.is_dir() {
        return Err(format!("no such directory: {}", path.display()).into());
    }

    Ok(FileObjects::new(path))
}

/// Load objects from a git+<scheme> URL.
///
/// The supplied `scheme` is an iterator over the current schemes (separated by `+`).
pub fn objects_from_git<'a, I>(
    config: ObjectsConfig,
    scheme: I,
    url: &'a Url,
) -> Result<Box<Objects>>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut scheme = scheme.into_iter();

    let sub_scheme = scheme.next().ok_or_else(|| {
        format!("bad scheme ({}), expected git+scheme", url.scheme())
    })?;

    let git_repo = git::setup_git_repo(&config.repo_dir, sub_scheme, url)?;

    let file_objects = FileObjects::new(git_repo.path());

    let git_repo = Rc::new(git_repo);
    let objects = GitObjects::new(url.clone(), git_repo, file_objects);

    Ok(Box::new(objects))
}

/// Load objects from an HTTP url.
pub fn objects_from_http(config: ObjectsConfig, url: &Url) -> Result<Box<Objects>> {
    let core = Core::new()?;

    let http_objects = HttpObjects::new(url.clone(), core);

    if let Some(cache_dir) = config.cache_dir {
        let missing_cache_time = config.missing_cache_time.unwrap_or_else(
            || Duration::new(60, 0),
        );

        return Ok(Box::new(CachedObjects::new(
            cache_dir,
            missing_cache_time,
            http_objects,
        )));
    }

    Ok(Box::new(http_objects))
}

/// Load objects from an URL.
pub fn objects_from_url(config: ObjectsConfig, url: &Url) -> Result<Box<Objects>> {
    let mut scheme = url.scheme().split("+");

    let first = scheme.next().ok_or_else(|| format!("bad scheme: {}", url))?;

    match first {
        "file" => objects_from_path(Path::new(url.path())).map(|o| Box::new(o) as Box<Objects>),
        "git" => objects_from_git(config, scheme, url),
        "http" => objects_from_http(config, url),
        scheme => Err(format!("bad scheme: {}", scheme).into()),
    }.chain_err(|| format!("load objects from url: {}", url))
}