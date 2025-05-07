pub mod auto_eat;

use azalea::app::{PluginGroup, PluginGroupBuilder};
use auto_eat::AutoEatPlugin;


pub struct ModulesPluginGroup;

impl PluginGroup for ModulesPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(AutoEatPlugin)
    }
}