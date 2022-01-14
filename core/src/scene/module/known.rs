use std::collections::HashMap;
use std::lazy::SyncLazy;

use crate::scene::module::clear::ClearModule;
use crate::scene::module::countdown::CountdownModule;
use crate::scene::module::image::ImageModule;
use crate::scene::module::snow::SnowModule;
use crate::scene::module::text::TextModule;
use crate::scene::module::{Module, ModuleWrapper};

// Ideally all of this would be one huge const initializer, but there are no const
// maps nor const ModuleWrappers yet.
static KNOWN_MODULES: SyncLazy<HashMap<String, ModuleWrapper>> = SyncLazy::new(|| {
    let mut map = HashMap::new();

    fn insert_helper<M: Module + 'static>(map: &mut HashMap<String, ModuleWrapper>) {
        map.insert(M::name(), ModuleWrapper::of::<M>());
    }

    insert_helper::<ClearModule>(&mut map);
    insert_helper::<TextModule>(&mut map);
    insert_helper::<SnowModule>(&mut map);
    insert_helper::<ImageModule>(&mut map);
    insert_helper::<CountdownModule>(&mut map);

    map
});

/// Helper for methods associated with retrieving modules.
pub struct KnownModules;

impl KnownModules {
    /// Retrieves an iterator for all entries
    pub fn iter() -> impl Iterator<Item = (&'static String, &'static ModuleWrapper)> {
        (*KNOWN_MODULES).iter()
    }

    /// Looks up a module by its name.
    pub fn look_up(name: &str) -> Option<&'static ModuleWrapper> {
        (*KNOWN_MODULES).get(name)
    }
}
