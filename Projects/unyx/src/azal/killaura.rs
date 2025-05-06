use azalea::{
    ecs::prelude::*,
    entity::{metadata::{AbstractAnimal, AbstractMonster}, Dead, LocalEntity, Position},
    prelude::*,
    pathfinder::goals::XZGoal,
    world::{InstanceName, MinecraftEntityId},
};

use crate::azal::State;

pub fn tick_mob_killaura(bot: Client, state: State) -> color_eyre::Result<()> {
    // println!("{}", state.mob_killaura);
    if !state.mob_killaura {
        return Ok(());
    }
    // println!("Killaura tick");
    if bot.has_attack_cooldown() {
        return Ok(());
    }
    
    let mut nearest_attackable_entity = None;
    let mut nearest_attackable_distance = f64::INFINITY;
    let mut nearest_targetable_entity = None;
    let mut nearest_targetable_distance = f64::INFINITY;
    let mut nearest_targetable_position = None;
    
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
            
            // If within attack range (4 blocks)
            if distance < 4.0 && distance < nearest_attackable_distance {
                nearest_attackable_entity = Some(entity_id);
                nearest_attackable_distance = distance;
            } 
            // If outside attack range but within pathfinding range (10 blocks)
            else if distance < 10.0 && distance < nearest_targetable_distance {
                nearest_targetable_entity = Some(entity_id);
                nearest_targetable_distance = distance;
                nearest_targetable_position = Some(position.clone());
            }
        }
    }
    
    // First priority: Attack if a mob is within attack range
    if let Some(nearest_entity) = nearest_attackable_entity {
        bot.attack(nearest_entity);
    } 
    // Second priority: Move towards a mob that's out of attack range but within pathfinding range
    else if let Some(position) = nearest_targetable_position {
        bot.goto(XZGoal { 
            x: position.x as i32, 
            z: position.z as i32 
        });
    }

    Ok(())
}