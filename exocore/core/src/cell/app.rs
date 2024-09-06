use std::{path::Path, sync::Arc};

use exocore_protos::{
    generated::exocore_apps::{manifest_schema::Source, Manifest},
    reflect::{FileDescriptorSet, Message},
};
use libp2p::PeerId;

use super::{Error, ManifestExt};
use crate::{
    dir::DynDirectory,
    sec::{
        hash::{multihash_decode_bs58, multihash_sha3_256, MultihashExt},
        keys::{Keypair, PublicKey},
    },
};

const MANIFEST_FILE_NAME: &str = "app.yaml";

/// Application that extends the capability of the cell by providing schemas and
/// WebAssembly logic.
#[derive(Clone)]
pub struct Application {
    identity: Arc<Identity>,
    schemas: Arc<[FileDescriptorSet]>,
    dir: DynDirectory,
}

struct Identity {
    public_key: PublicKey,
    id: ApplicationId,
    manifest: Manifest,
}

impl Application {
    pub fn generate(
        dir: impl Into<DynDirectory>,
        name: String,
    ) -> Result<(Keypair, Application), Error> {
        let keypair = Keypair::generate_ed25519();
        let dir = dir.into();

        let manifest = Manifest {
            name,
            public_key: keypair.public().encode_base58_string(),
            version: "0.0.1".to_string(),
            ..Default::default()
        };

        Ok((keypair, Application::from_manifest(dir, manifest)?))
    }

    pub fn from_directory(dir: impl Into<DynDirectory>) -> Result<Application, Error> {
        let dir = dir.into();

        let manifest = {
            let manifest_file = dir.open_read(Path::new(MANIFEST_FILE_NAME))?;
            Manifest::read_yaml(manifest_file)?
        };

        Self::from_manifest(dir, manifest)
    }

    pub fn from_manifest(
        dir: impl Into<DynDirectory>,
        manifest: Manifest,
    ) -> Result<Application, Error> {
        let dir = dir.into();
        let public_key = PublicKey::decode_base58_string(&manifest.public_key).map_err(|err| {
            Error::Application(
                manifest.name.clone(),
                anyhow!("Error parsing application public_key: {}", err),
            )
        })?;

        let id = ApplicationId::from_public_key(&public_key);

        let mut schemas = Vec::new();
        for app_schema in &manifest.schemas {
            match &app_schema.source {
                Some(Source::File(schema_path)) => {
                    let schema_file = dir.open_read(Path::new(schema_path))?;
                    let fd_set = read_file_descriptor_set(&manifest.name, schema_file)?;
                    schemas.push(fd_set);
                }
                other => {
                    return Err(Error::Application(
                        manifest.name.clone(),
                        anyhow!("Unsupported application schema source: {:?}", other),
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
            schemas: schemas.into(),
            dir,
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

    pub fn version(&self) -> &str {
        &self.identity.manifest.version
    }

    pub fn manifest(&self) -> &Manifest {
        &self.identity.manifest
    }

    pub fn schemas(&self) -> &[FileDescriptorSet] {
        self.schemas.as_ref()
    }

    pub fn directory(&self) -> &DynDirectory {
        &self.dir
    }

    pub fn validate(&self) -> Result<(), Error> {
        // validate module
        if let Some(module) = &self.manifest().module {
            let module_file = self.directory().open_read(Path::new(&module.file))?;

            let module_multihash = multihash_sha3_256(module_file).map_err(|err| {
                Error::Application(
                    self.name().to_string(),
                    anyhow!("Couldn't multihash module file at: {}", err),
                )
            })?;

            let expected_multihash =
                multihash_decode_bs58::<32>(&module.multihash).map_err(|err| {
                    Error::Application(
                        self.name().to_string(),
                        anyhow!(
                            "{}: Couldn't decode expected module multihash in manifest: {}",
                            self.name(),
                            err
                        ),
                    )
                })?;

            if expected_multihash != module_multihash {
                return Err(Error::Application(
                    self.name().to_string(),
                    anyhow!(
                        "Module multihash in manifest doesn't match module file (expected={} module={})",
                        expected_multihash.encode_bs58(),
                        module_multihash.encode_bs58(),
                    ),
                ));
            }
        }

        Ok(())
    }

    pub fn save_manifest(&self, manifest: &Manifest) -> Result<(), Error> {
        let manifest_file = self.dir.open_write(Path::new(MANIFEST_FILE_NAME))?;
        manifest.write_yaml(manifest_file)?;
        Ok(())
    }

    pub fn manifest_exists(dir: impl Into<DynDirectory>) -> bool {
        dir.into().exists(Path::new(MANIFEST_FILE_NAME))
    }
}

/// Unique identifier of an application, which is built by hashing the public
/// key.
///
/// For now, this ID is generated the same way as node IDs.
#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub struct ApplicationId(String);

impl ApplicationId {
    /// Create a Cell ID from a public key by using libp2p method to be
    /// compatible with it
    pub fn from_public_key(public_key: &PublicKey) -> ApplicationId {
        let peer_id = PeerId::from_public_key(public_key.to_libp2p());
        ApplicationId(peer_id.to_string())
    }

    pub fn from_base58_public_key(public_key: &str) -> Result<ApplicationId, Error> {
        let public_key = PublicKey::decode_base58_string(public_key)?;
        Ok(ApplicationId::from_public_key(&public_key))
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

fn read_file_descriptor_set(
    app_name: &str,
    mut content: impl std::io::Read,
) -> Result<FileDescriptorSet, Error> {
    let fd_set = FileDescriptorSet::parse_from_reader(&mut content).map_err(|err| {
        Error::Application(
            app_name.to_string(),
            anyhow!(
                "Couldn't parse application schema file descriptor set: {}",
                err
            ),
        )
    })?;

    Ok(fd_set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dir::ram::RamDirectory;

    #[test]
    fn generate_and_validate() -> anyhow::Result<()> {
        let dir = RamDirectory::new();
        let (_kp, app) = Application::generate(dir, "some_app".to_string())?;
        app.validate()?;
        Ok(())
    }

    #[test]
    fn app_id_conversion() {
        let kp = crate::sec::keys::Keypair::generate_ed25519();
        let app_id = ApplicationId::from_public_key(&kp.public());

        assert_eq!(app_id, ApplicationId::from_string(app_id.to_string()));
        assert_eq!(app_id, ApplicationId::from_bytes(app_id.as_bytes()));
        assert_eq!(
            app_id,
            ApplicationId::from_base58_public_key(kp.public().encode_base58_string().as_str())
                .unwrap()
        );
    }
}
