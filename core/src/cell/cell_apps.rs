use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, RwLock},
};

use exocore_protos::{
    generated::exocore_core::{cell_application_config, CellApplicationConfig},
    registry::Registry,
};

use super::{Application, ApplicationId, Error};

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

    pub(crate) fn load_from_cell_apps_conf<'c, I>(&self, iter: I) -> Result<(), Error>
    where
        I: Iterator<Item = &'c CellApplicationConfig> + 'c,
    {
        for cell_app in iter {
            let app_location = if let Some(loc) = &cell_app.location {
                loc
            } else {
                warn!(
                    "Cannot load application {} (version {}). No location configured.",
                    cell_app.name, cell_app.version
                );
                continue;
            };

            match app_location {
                cell_application_config::Location::Inline(manifest) => {
                    let application = Application::new_from_manifest(manifest.clone())?;
                    self.add_application(application)?;
                }
                cell_application_config::Location::Path(dir) => {
                    let application = Application::new_from_directory(&dir)?;
                    self.add_application(application)?;
                }
            }
        }

        Ok(())
    }

    pub fn add_application(&self, application: Application) -> Result<(), Error> {
        let mut apps = self.applications.write().unwrap();

        for fd_set in application.schemas() {
            self.schemas.register_file_descriptor_set(fd_set);
        }

        apps.insert(application.id().clone(), CellApplication { application });
        Ok(())
    }

    pub fn applications(&self) -> Vec<CellApplication> {
        let apps = self.applications.read().expect("couldn't lock inner");
        apps.values().cloned().collect()
    }
}

#[derive(Clone)]
pub struct CellApplication {
    application: Application,
}

impl CellApplication {
    pub fn application(&self) -> &Application {
        &self.application
    }
}

impl Deref for CellApplication {
    type Target = Application;

    fn deref(&self) -> &Self::Target {
        &self.application
    }
}
