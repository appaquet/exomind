use super::{app_manifest_from_yaml_file, Error};
use crate::protos::generated::exocore_apps::manifest_schema::Source;
use crate::protos::generated::exocore_apps::Manifest;
use crate::{sec::keys::PublicKey, utils::path::child_to_abs_path};
use protobuf::descriptor::FileDescriptorSet;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

/// Application that extends the capability of the cell by providing schemas and
/// WebAssembly logic.
#[derive(Clone)]
pub struct Application {
    identity: Arc<Identity>,
    schemas: Arc<Vec<FileDescriptorSet>>,
}

struct Identity {
    public_key: PublicKey,
    id: ApplicationId,
    manifest: Manifest,
}

impl Application {
    pub fn new_from_directory<P: AsRef<Path>>(dir: P) -> Result<Application, Error> {
        let mut manifest_path = dir.as_ref().to_path_buf();
        manifest_path.push("app.yaml");

        let mut manifest = app_manifest_from_yaml_file(manifest_path)?;
        manifest.path = dir.as_ref().to_string_lossy().to_string();

        Self::build(manifest)
    }

    pub fn new_from_manifest(manifest: Manifest) -> Result<Application, Error> {
        Self::build(manifest)
    }

    fn build(manifest: Manifest) -> Result<Application, Error> {
        let public_key = PublicKey::decode_base58_string(&manifest.public_key).map_err(|err| {
            Error::Application(
                manifest.name.clone(),
                format!("Error parsing application public_key: {}", err),
            )
        })?;

        let id = ApplicationId::from_public_key(&public_key);

        let mut schemas = Vec::new();
        for app_schema in &manifest.schemas {
            match &app_schema.source {
                Some(Source::File(rel_path)) => {
                    let schema_path = child_to_abs_path(&manifest.path, rel_path);
                    let fd_set = read_file_descriptor_set_file(&manifest.name, schema_path)?;
                    schemas.push(fd_set);
                }
                Some(Source::Bytes(bytes)) => {
                    let bytes = bytes.as_slice();
                    let schema = protobuf::parse_from_bytes(bytes).map_err(|err| {
                        Error::Application(
                            manifest.name.clone(),
                            format!(
                                "Couldn't parse application schema file descriptor set: {}",
                                err
                            ),
                        )
                    })?;

                    schemas.push(schema)
                }
                other => {
                    return Err(Error::Application(
                        manifest.name.clone(),
                        format!("Unsupported application schema source: {:?}", other),
                    ));
                }
            }
        }

        Ok(Application {
            identity: Arc::new(Identity {
                public_key,
                id,
                manifest,
            }),
            schemas: Arc::new(schemas),
        })
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.identity.public_key
    }

    pub fn id(&self) -> &ApplicationId {
        &self.identity.id
    }

    pub fn name(&self) -> &str {
        &self.identity.manifest.name
    }

    pub fn manifest(&self) -> &Manifest {
        &self.identity.manifest
    }

    pub fn schemas(&self) -> &[FileDescriptorSet] {
        self.schemas.as_slice()
    }
}

/// Unique identifier of an application, which is built by hashing the public
/// key
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct ApplicationId(String);

impl ApplicationId {
    pub fn from_public_key(public_key: &PublicKey) -> ApplicationId {
        let id = public_key.encode_base58_string();
        ApplicationId(id)
    }

    pub fn from_string(id: String) -> ApplicationId {
        ApplicationId(id)
    }

    pub fn from_bytes(id: &[u8]) -> ApplicationId {
        ApplicationId(String::from_utf8_lossy(id).to_string())
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl std::fmt::Display for ApplicationId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::str::FromStr for ApplicationId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ApplicationId(s.to_string()))
    }
}

fn read_file_descriptor_set_file<P: AsRef<Path>>(
    app_name: &str,
    path: P,
) -> Result<FileDescriptorSet, Error> {
    let mut file = File::open(path).map_err(|err| {
        Error::Application(
            app_name.to_string(),
            format!(
                "Couldn't open application file descriptor set file: {}",
                err
            ),
        )
    })?;

    let fdset = protobuf::parse_from_reader(&mut file).map_err(|err| {
        Error::Application(
            app_name.to_string(),
            format!(
                "Couldn't parse application schema file descriptor set: {}",
                err
            ),
        )
    })?;

    Ok(fdset)
}
