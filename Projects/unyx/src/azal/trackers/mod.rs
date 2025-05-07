use azalea::app::{PluginGroup, PluginGroupBuilder};

pub mod game_tick;

pub TrackersGroup;

impl PluginGroup for TrackersGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(game_tick::GameTickPlugin)
    }
}