pub mod auto_eat;

use azalea::app::{PluginGroup, PluginGroupBuilder};

pub struct ModulesPluginGroup;

impl PluginGroup for ModulesPluginGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(AntiAfkPlugin)
            .add(AutoEatPlugin)
            .add(AutoLeavePlugin)
            .add(AutoKillPlugin)
            .add(AutoLookPlugin)
            .add(AutoPearlPlugin)
            .add(AutoTotemPlugin)
            .add(AutoWhitelistPlugin)
    }
}