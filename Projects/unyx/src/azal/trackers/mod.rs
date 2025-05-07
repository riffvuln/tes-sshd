use azalea::app::{PluginGroup, PluginGroupBuilder};

pub mod game_tick;

pub struct TrackersGroup;

impl PluginGroup for TrackersGroup {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(game_tick::GameTickPlugin)
    }
}