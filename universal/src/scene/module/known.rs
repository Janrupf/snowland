use std::collections::HashMap;
use std::lazy::SyncLazy;

use crate::scene::module::ModuleWrapper;

// Ideally all of this would be one huge const initializer, but there are no const
// maps nor const ModuleWrappers yet.
static KNOWN_MODULES: SyncLazy<HashMap<String, ModuleWrapper>> = SyncLazy::new(|| {
    let map = HashMap::new();

    map
});

/// Helper for methods associated with retrieving modules.
pub struct KnownModules;

impl KnownModules {
    /// Retrieves all known modules.
    pub fn all() -> &'static HashMap<String, ModuleWrapper> {
        &*KNOWN_MODULES
    }

    /// Looks up a module by its name.
    pub fn look_up(name: &str) -> Option<&'static ModuleWrapper> {
        (*KNOWN_MODULES).get(name)
    }
}
