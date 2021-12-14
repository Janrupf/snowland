use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::scene::module::{
    KnownModules, Module, ModuleConfigError, ModuleContainer, ModuleWrapper, ModuleWrapperPair,
};

/// Helper representing the entire config structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigIO {
    modules: Vec<ModuleConfigPair>,
}

/// Structure representing a mapping of a module's type to its configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleConfigPair {
    pub ty: String,
    pub config: serde_json::Value,
}

impl ConfigIO {
    /// Saves the modules to the configuration.
    pub fn save<'a>(
        modules: impl Iterator<Item = &'a Box<dyn ModuleContainer>>,
    ) -> Result<(), ModuleConfigError> {
        let modules = modules
            .map(|c| {
                let ty = c.module_type();
                let config = c.serialize_config();

                config.map(|config| ModuleConfigPair { ty, config })
            })
            .collect::<Result<Vec<ModuleConfigPair>, ModuleConfigError>>()?;

        let io = Self { modules };
        let writer = Self::open(true)?;

        serde_json::to_writer_pretty(writer, &io).map_err(ModuleConfigError::Serialize)
    }

    /// Loads the modules from the configuration.
    pub fn load() -> Result<Vec<ModuleWrapperPair>, ModuleConfigError> {
        let reader = Self::open(false)?;
        let configs =
            serde_json::from_reader::<_, Self>(reader).map_err(ModuleConfigError::Deserialize)?;

        let modules = configs
            .modules
            .into_iter()
            .filter_map(|ModuleConfigPair { ty, config }| {
                log::debug!("Loading module of type {}", ty);
                log::trace!("for {} config = {}", ty, config);

                match KnownModules::look_up(&ty).map(|w| w.deserialize_from_config(config)) {
                    None => {
                        log::warn!("Skipping module of unknown type {}", ty);
                        None
                    }
                    Some(Err(err)) => {
                        log::warn!(
                            "Failed to deserialize config for module of type {}: {}",
                            ty,
                            err
                        );
                        None
                    }
                    Some(Ok(module)) => Some(module),
                }
            })
            .collect();

        Ok(modules)
    }

    /// Opens the modules file for access.
    fn open(for_writing: bool) -> Result<File, ModuleConfigError> {
        let path = "./modules.json";

        if for_writing {
            File::create(path)
        } else {
            File::open(path)
        }
        .map_err(Into::into)
    }
}
