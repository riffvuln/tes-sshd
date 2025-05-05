use azalea::{
    ecs::prelude::*,
    entity::{metadata::{AbstractAnimal, AbstractMonster}, Dead, LocalEntity, Position},
    prelude::*,
    world::{InstanceName, MinecraftEntityId},
};

use crate::azal::State;

pub fn tick_mob_killaura(bot: Client, state: State) -> color_eyre::Result<()> {
    println!("{state:?}");
    if !state.mob_killaura {
        return Ok(());
    }
    println!("Killaura tick");
    if bot.has_attack_cooldown() {
        return Ok(());
    }
    let mut nearest_entity = None;
    let mut nearest_distance = f64::INFINITY;
    let bot_position = bot.eye_position();
    let bot_instance_name = bot.component::<InstanceName>();
    {
        let mut ecs = bot.ecs.lock();
        let mut query = ecs
            .query_filtered::<(&MinecraftEntityId, &Position, &InstanceName), (
                With<AbstractMonster>,
                // With<AbstractAnimal>,
                Without<LocalEntity>,
                Without<Dead>,
            )>();
        for (&entity_id, position, instance_name) in query.iter(&ecs) {
            if instance_name != &bot_instance_name {
                continue;
            }

            let distance = bot_position.distance_to(position);
            if distance < 4. && distance < nearest_distance {
                nearest_entity = Some(entity_id);
                nearest_distance = distance;
            }
        }
    }
    if let Some(nearest_entity) = nearest_entity {
        bot.attack(nearest_entity);
    }

    Ok(())
}