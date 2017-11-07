use super::imports::*;
use core::{Object, Version};
use std::fmt;

/// Candidate to publish.
struct Match(Version, Box<Object>, RpPackage);

/// Formatting of candidate.
struct DisplayMatch<'a>(&'a Match);

impl<'a> fmt::Display for DisplayMatch<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner = &self.0;
        write!(f, "{}@{}", inner.1, inner.0)
    }
}

pub fn options<'a, 'b>() -> App<'a, 'b> {
    let out = SubCommand::with_name("publish").about("Publish specifications");

    let out = out.arg(Arg::with_name("force").long("force").help(
        "Force a publish, \
         even if it already \
         exists",
    ));

    let out = out.arg(Arg::with_name("package").multiple(true));

    out
}

pub fn entry(matches: &ArgMatches) -> Result<()> {
    let manifest = setup_manifest(matches)?;
    let mut repository = setup_repository(&manifest.repository)?;

    let mut resolver = setup_path_resolver(&manifest)?.ok_or_else(|| {
        "could not setup path resolver"
    })?;

    let packages: Vec<RpRequiredPackage> = matches
        .values_of("package")
        .into_iter()
        .flat_map(|it| it)
        .map(|p| RpRequiredPackage::parse(p).map_err(Into::into))
        .collect::<Result<_>>()?;

    let mut results = Vec::new();

    for publish in manifest.publish {
        let package = RpRequiredPackage::new(publish.package.clone(), None);
        let resolved = resolver.resolve(&package)?;

        if resolved.is_empty() {
            return Err(
                format!("no matching packages found for: {}", package).into(),
            );
        }

        // packages.push(RpRequiredPackage());
        for (_, object) in resolved {
            results.push(Match(
                publish.version.clone(),
                object,
                publish.package.clone(),
            ));
        }
    }

    for package in packages.iter() {
        let resolved = resolver.resolve(package)?;

        if resolved.is_empty() {
            return Err(
                format!("no matching packages found for: {}", package).into(),
            );
        }

        for (version, object) in resolved {
            let version = version.ok_or_else(
                || format!("{}: package without a version", object),
            )?;

            results.push(Match(version, object, package.package.clone()));
        }
    }

    let mut it = results.into_iter();

    let first = it.next().ok_or_else(|| format!("no packages to publish"))?;

    if let Some(next) = it.next() {
        warn!("matched: {}", DisplayMatch(&first));
        warn!("    and: {}", DisplayMatch(&next));

        while let Some(next) = it.next() {
            warn!("    and: {}", DisplayMatch(&next));
        }

        return Err("more than one matching package found".into());
    }

    let Match(version, object, package) = first;

    info!("publishing: {}@{} (from {})", package, version, object);

    let force = matches.is_present("force");
    repository.publish(&object, &package, &version, force)?;
    Ok(())
}
