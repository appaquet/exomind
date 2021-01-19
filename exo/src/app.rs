use std::fs::File;
use std::io::{BufWriter, Cursor, Read, Seek};
use std::path::{Path, PathBuf};

use clap::Clap;
use exocore_core::cell::{Application, ManifestExt};
use exocore_core::protos::apps::Manifest;
use zip::write::FileOptions;

use crate::term::{print_success, style_value};
use crate::utils::expand_tild;
use crate::Context;

#[derive(Clap)]
pub struct AppOptions {
    #[clap(subcommand)]
    pub command: AppCommand,
}

#[derive(Clap)]
pub enum AppCommand {
    /// Package an application.
    Package(PackageOptions),
}

#[derive(Clap)]
pub struct PackageOptions {
    directory: Option<PathBuf>,
}

pub async fn handle_cmd(ctx: &Context, app_opts: &AppOptions) -> anyhow::Result<()> {
    match &app_opts.command {
        AppCommand::Package(pkg_opts) => cmd_package(ctx, app_opts, pkg_opts),
    }
}

fn cmd_package(
    _ctx: &Context,
    _app_opts: &AppOptions,
    pkg_opts: &PackageOptions,
) -> anyhow::Result<()> {
    let cur_dir = std::env::current_dir().expect("Couldn't get current directory");

    let app_dir = pkg_opts
        .directory
        .clone()
        .unwrap_or_else(|| cur_dir.clone());
    let app_dir = expand_tild(app_dir).expect("Couldn't expand app directory");

    let manifest_path = app_dir.join("app.yaml");
    let manifest_abs =
        Manifest::from_yaml_file(manifest_path).expect("Couldn't read manifest file");

    // for now we inline the manifest so that it's easier to read, but at some point
    // should write dependencies to zip
    let mut manifest_zip = manifest_abs.inlined().expect("Couldn't inline manifest");
    manifest_zip.make_relative_paths(&app_dir);

    let app_name = manifest_abs.name;

    let zip_file_path = cur_dir.join(format!("{}.zip", app_name));
    let zip_file = File::create(&zip_file_path).expect("Couldn't create zip file");
    let zip_file_buf = BufWriter::new(zip_file);
    let mut zip_archive = zip::ZipWriter::new(zip_file_buf);

    zip_archive.start_file("app.yaml", FileOptions::default())?;
    manifest_zip
        .to_yaml_writer(&mut zip_archive)
        .expect("Couldn't write manifest to zip");
    zip_archive.finish()?;

    print_success(format!(
        "Application {} version {} got packaged to {}",
        style_value(app_name),
        style_value(manifest_abs.version),
        style_value(zip_file_path),
    ));

    Ok(())
}

pub async fn fetch_package_url<U: Into<url::Url>>(url: U) -> anyhow::Result<Application> {
    let fetch_resp = reqwest::get(url.into())
        .await
        .map_err(|err| anyhow!("Couldn't fetch package: {}", err))?;

    let package_bytes = fetch_resp
        .bytes()
        .await
        .map_err(|err| anyhow!("Couldn't fetch bytes for package: {}", err))?;

    let cursor = Cursor::new(package_bytes.as_ref());
    read_package(cursor)
}

pub fn read_package_path<P: AsRef<Path>>(path: P) -> anyhow::Result<Application> {
    let file =
        File::open(path.as_ref()).map_err(|err| anyhow!("Couldn't open package file: {}", err))?;

    read_package(file)
}

pub fn read_package<R: Read + Seek>(reader: R) -> anyhow::Result<Application> {
    let mut package_zip = zip::ZipArchive::new(reader)
        .map_err(|err| anyhow!("Couldn't read package zip: {}", err))?;

    let manifest = package_zip
        .by_name("app.yaml")
        .map_err(|err| anyhow!("Couldn't find 'app.yaml' manifest in package: {}", err))?;

    let manifest = Manifest::from_yaml(manifest)
        .map_err(|err| anyhow!("Couldn't read manifest from package: {}", err))?;

    let app = Application::new_from_manifest(manifest)
        .map_err(|err| anyhow!("Couldn't create app from manifest: {}", err))?;

    Ok(app)
}
