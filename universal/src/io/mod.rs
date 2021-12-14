use std::fs::File;

use serde::{Deserialize, Serialize};

use crate::scene::module::{ModuleConfigError, ModuleContainer};

/// Helper representing the entire config structure.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigIO {
    modules: Vec<ModuleConfigPair>,
}

/// Structure representing a mapping of a module's type to its configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleConfigPair {
    ty: String,
    config: serde_json::Value,
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
