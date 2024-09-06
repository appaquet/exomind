use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use exocore_protos::{generated::exocore_core::CellApplicationConfig, registry::Registry};

use super::{Application, ApplicationId, CellId, Error};
use crate::{dir::DynDirectory, sec::keys::PublicKey};

/// Applications installed in a cell.
#[derive(Clone)]
pub struct CellApplications {
    applications: Arc<RwLock<HashMap<ApplicationId, CellApplication>>>,
    schemas: Arc<Registry>,
}

impl CellApplications {
    pub(crate) fn new(schemas: Arc<Registry>) -> CellApplications {
        CellApplications {
            applications: Arc::new(RwLock::new(HashMap::new())),
            schemas,
        }
    }

    pub(crate) fn load_from_configurations<'c, I>(
        &self,
        cell_id: &CellId,
        apps_dir: &DynDirectory,
        iter: I,
    ) -> Result<(), Error>
    where
        I: Iterator<Item = &'c CellApplicationConfig> + 'c,
    {
        for cell_app in iter {
            let app_id = ApplicationId::from_base58_public_key(&cell_app.public_key)?;
            let app_dir = cell_app_directory(apps_dir, &app_id, &cell_app.version);
            let app_pk = PublicKey::decode_base58_string(&cell_app.public_key)?;

            if Application::manifest_exists(app_dir.clone()) {
                info!(
                    "{}: Adding loaded application '{}' (id='{}')",
                    cell_id, cell_app.name, app_id
                );
                let app = Application::from_directory(app_dir).map_err(|err| {
                    Error::Application(
                        cell_app.name.clone(),
                        anyhow!("failed to load from directory: {}", err),
                    )
                })?;
                self.add_loaded_application(cell_app.clone(), app)?;
            } else {
                info!(
                    "{}: Adding unloaded application '{}' (id='{}')",
                    cell_id, cell_app.name, app_id
                );
                self.add_unloaded_application(app_id, app_pk, cell_app.clone())?;
            }
        }

        Ok(())
    }

    fn add_loaded_application(
        &self,
        cell_app_config: CellApplicationConfig,
        application: Application,
    ) -> Result<(), Error> {
        let mut apps = self.applications.write().unwrap();

        for fd_set in application.schemas() {
            self.schemas.register_file_descriptor_set(fd_set);
        }

        apps.insert(
            application.id().clone(),
            CellApplication {
                id: application.id().clone(),
                name: application.name().to_string(),
                version: application.version().to_string(),
                public_key: application.public_key().clone(),
                application: Some(application),
                package_url: cell_app_config.package_url,
            },
        );
        Ok(())
    }

    fn add_unloaded_application(
        &self,
        id: ApplicationId,
        public_key: PublicKey,
        cell_app: CellApplicationConfig,
    ) -> Result<(), Error> {
        let mut apps = self.applications.write().unwrap();

        apps.insert(
            id.clone(),
            CellApplication {
                id,
                name: cell_app.name,
                version: cell_app.version,
                public_key,
                application: None,
                package_url: cell_app.package_url,
            },
        );
        Ok(())
    }

    pub fn get(&self) -> Vec<CellApplication> {
        let apps = self.applications.read().expect("couldn't lock inner");
        apps.values().cloned().collect()
    }
}

#[derive(Clone)]
pub struct CellApplication {
    id: ApplicationId,
    name: String,
    version: String,
    public_key: PublicKey,
    application: Option<Application>,
    package_url: String,
}

impl CellApplication {
    pub fn id(&self) -> &ApplicationId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn package_url(&self) -> &str {
        self.package_url.as_str()
    }

    pub fn is_loaded(&self) -> bool {
        self.application.is_some()
    }

    pub fn get(&self) -> Option<&Application> {
        self.application.as_ref()
    }
}

pub fn cell_app_directory(
    apps_dir: &DynDirectory,
    app_id: &ApplicationId,
    app_version: &str,
) -> DynDirectory {
    apps_dir.scope(PathBuf::from(format!("{}_{}", app_id, app_version)))
}
