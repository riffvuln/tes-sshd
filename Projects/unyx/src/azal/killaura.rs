use azalea::{
    ecs::prelude::*,
    entity::{Dead, LocalEntity, Position, metadata::AbstractMonster},
    prelude::*,
    world::{InstanceName, MinecraftEntityId},
};

use crate::azal::State;

pub fn tick_mob_killaura()