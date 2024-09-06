use std::{
    fs::File,
    io::{BufWriter, Cursor, Read, Seek},
    path::{Path, PathBuf},
};

use exocore_core::{
    cell::{Application, Cell, CellApplicationConfigExt, CellConfigExt, ManifestExt},
    dir::os::OsDirectory,
    sec::{
        hash::{multihash_sha3_256_file, MultihashExt},
        keys::Keypair,
    },
};
use exocore_protos::{apps::Manifest, core::CellApplicationConfig};
use tempfile::{tempdir_in, TempDir};
use zip::write::SimpleFileOptions;

use crate::{
    term::{print_action, print_error, print_info, print_success, print_warning, style_value},
    utils::expand_tild,
    Context,
};

#[derive(clap::Parser)]
pub struct AppOptions {
    #[clap(subcommand)]
    pub command: AppCommand,
}

#[derive(clap::Parser)]
pub enum AppCommand {
    /// Generates an application structure in current directory.
    Generate(GenerateOptions),

    /// Packages an application.
    Package(PackageOptions),
}

#[derive(clap::Parser)]
pub struct GenerateOptions {
    /// Application name
    name: String,
}

#[derive(clap::Parser)]
pub struct PackageOptions {
    directory: Option<PathBuf>,
}

pub async fn handle_cmd(ctx: &Context, app_opts: &AppOptions) {
    match &app_opts.command {
        AppCommand::Generate(gen_opts) => cmd_generate(ctx, app_opts, gen_opts),
        AppCommand::Package(pkg_opts) => cmd_package(ctx, app_opts, pkg_opts),
    }
}

fn cmd_generate(_ctx: &Context, _app_opts: &AppOptions, gen_opts: &GenerateOptions) {
    let cur_dir = std::env::current_dir().expect("Couldn't get current directory");

    let kp = Keypair::generate_ed25519();

    let manifest = Manifest {
        name: gen_opts.name.clone(),
        version: "0.0.1".to_string(),
        public_key: kp.public().encode_base58_string(),
        schemas: Vec::new(),
        module: None,
    };

    let manifest_path = cur_dir.join("app.yaml");
    let manifest_file = File::create(manifest_path).expect("Couldn't create manifest file");
    manifest
        .write_yaml(manifest_file)
        .expect("Couldn't write manifest");

    print_success(format!(
        "Application {} generated.",
        style_value(&gen_opts.name)
    ));
    print_warning(format!(
        "Application keypair (to be saved securely!): {}",
        style_value(kp.encode_base58_string())
    ));
}

fn cmd_package(_ctx: &Context, _app_opts: &AppOptions, pkg_opts: &PackageOptions) {
    let cur_dir = std::env::current_dir().expect("Couldn't get current directory");

    let app_dir = pkg_opts
        .directory
        .clone()
        .unwrap_or_else(|| cur_dir.clone());
    let app_dir = expand_tild(app_dir).expect("Couldn't expand app directory");

    let manifest_path = app_dir.join("app.yaml");
    let manifest_file = File::open(manifest_path).expect("Couldn't open manifest file");
    let mut manifest = Manifest::read_yaml(manifest_file).expect("Couldn't read manifest file");

    if let Some(module) = &mut manifest.module {
        module.multihash = multihash_sha3_256_file(&module.file)
            .expect("Couldn't multihash module")
            .encode_bs58();
    }

    let zip_file_path = cur_dir.join(format!("{}.zip", manifest.name));
    let zip_file = File::create(&zip_file_path).expect("Couldn't create zip file");
    let zip_file_buf = BufWriter::new(zip_file);

    let mut zip_archive = zip::ZipWriter::new(zip_file_buf);

    zip_archive
        .start_file("app.yaml", SimpleFileOptions::default())
        .expect("Couldn't start zip file");
    manifest
        .write_yaml(&mut zip_archive)
        .expect("Couldn't write manifest to zip");

    if let Some(module) = &manifest.module {
        zip_archive
            .start_file(&module.file, SimpleFileOptions::default())
            .expect("Couldn't start zip file");

        let mut module_file = File::open(&module.file).expect("Couldn't open app module");
        std::io::copy(&mut module_file, &mut zip_archive).expect("Couldn't copy app module to zip");
    }

    for schema in &manifest.schemas {
        if let Some(exocore_protos::apps::manifest_schema::Source::File(file)) = &schema.source {
            zip_archive
                .start_file(file, SimpleFileOptions::default())
                .expect("Couldn't start zip file");

            let abs_file = app_dir.join(file);
            let mut schema_file = File::open(abs_file).expect("Couldn't open app schema");
            std::io::copy(&mut schema_file, &mut zip_archive)
                .expect("Couldn't copy app schema to zip");
        }
    }

    zip_archive.finish().expect("Couldn't finished zip file");

    print_success(format!(
        "Application {} version {} got packaged to {}",
        style_value(manifest.name),
        style_value(manifest.version),
        style_value(zip_file_path),
    ));
}

pub struct AppPackage {
    pub url: String,
    pub app: Application,
    pub temp_dir: TempDir,
}

impl AppPackage {
    pub async fn fetch_package_url<U: Into<String>>(
        cell: &Cell,
        url: U,
    ) -> anyhow::Result<AppPackage> {
        let url: String = url.into();
        if let Some(path) = url.strip_prefix("file://") {
            return Self::read_package_path(cell, path);
        }

        let fetch_resp = reqwest::get(url.clone())
            .await
            .map_err(|err| anyhow!("Couldn't fetch package: {}", err))?;

        let package_bytes = fetch_resp
            .bytes()
            .await
            .map_err(|err| anyhow!("Couldn't fetch bytes for package: {}", err))?;

        let cursor = Cursor::new(package_bytes.as_ref());

        let mut pkg = Self::read_package(cell, cursor)?;
        pkg.url = url;
        Ok(pkg)
    }

    pub fn read_package_path<P: AsRef<Path>>(cell: &Cell, path: P) -> anyhow::Result<AppPackage> {
        let file = File::open(path.as_ref())
            .map_err(|err| anyhow!("Couldn't open package file: {}", err))?;

        let mut pkg = Self::read_package(cell, file)?;
        pkg.url = format!("file://{}", path.as_ref().to_string_lossy());
        Ok(pkg)
    }

    fn read_package<R: Read + Seek>(cell: &Cell, reader: R) -> anyhow::Result<AppPackage> {
        let mut package_zip = zip::ZipArchive::new(reader)
            .map_err(|err| anyhow!("Couldn't read package zip: {}", err))?;

        let cell_temp_dir = cell
            .temp_directory()
            .as_os_path()
            .expect("Cell is not stored in an OS directory");
        std::fs::create_dir_all(&cell_temp_dir).expect("Couldn't create temp directory");

        let dir = tempdir_in(cell_temp_dir)
            .map_err(|err| anyhow!("Couldn't create temp dir: {}", err))?;

        package_zip
            .extract(dir.path())
            .map_err(|err| anyhow!("Couldn't extract package: {}", err))?;

        let app_dir = OsDirectory::new(dir.path().to_path_buf());
        let app = Application::from_directory(app_dir)
            .map_err(|err| anyhow!("Couldn't create app from manifest: {}", err))?;

        Ok(AppPackage {
            url: String::new(),
            app,
            temp_dir: dir,
        })
    }

    pub async fn install(&self, cell: &Cell, overwrite: bool) -> anyhow::Result<()> {
        let apps_dir = cell.apps_directory();
        let apps_dir_path = apps_dir.as_os_path()?;
        std::fs::create_dir_all(apps_dir_path).expect("Couldn't create app dir");

        let app_dir = cell.app_directory(self.app.manifest()).unwrap();
        let app_dir_path = app_dir.as_os_path()?;

        let temp_dir = OsDirectory::new(self.temp_dir.path().to_path_buf());

        let application = Application::from_directory(temp_dir)?;
        application
            .validate()
            .expect("Failed to validate the application");

        if app_dir_path.exists() {
            if overwrite {
                print_info(format!(
                    "Application already installed at '{}'. Overwriting it.",
                    style_value(&app_dir_path)
                ));
                std::fs::remove_dir_all(&app_dir_path).expect("Couldn't remove existing app dir");
            } else {
                print_error(format!(
                    "Application already installed at '{}'. Use {} to overwrite it.",
                    style_value(&app_dir_path),
                    style_value("--overwrite"),
                ));
                return Ok(());
            }
        }

        std::fs::rename(&self.temp_dir, &app_dir_path).expect("Couldn't move temp app dir");

        let mut cell_app_config = CellApplicationConfig::from_manifest(self.app.manifest().clone());
        cell_app_config.location = None;
        cell_app_config.package_url.clone_from(&self.url);

        print_action("Writing cell config...");
        let mut cell_config = cell.config().clone();
        cell_config.add_application(cell_app_config);
        cell.save_config(&cell_config)
            .expect("Couldn't write cell config");

        Ok(())
    }
}
