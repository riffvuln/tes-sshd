use std::{cmp::Ordering, collections::HashMap, sync::LazyLock};

use azalea::{
    app::{App, Plugin},
    ecs::prelude::*,
    entity::{metadata::Player, LocalEntity, LookDirection},
    inventory::{
        operations::{ClickOperation, SwapClick},
        ContainerClickEvent,
        Inventory,
        InventorySet,
    },
    mining::continue_mining_block,
    packet::game::{handle_outgoing_packets, SendPacketEvent},
    physics::PhysicsSet,
    prelude::*,
    protocol::packets::game::{
        s_interact::InteractionHand,
        ServerboundGamePacket,
        ServerboundUseItem,
    },
    registry::Item,
    Hunger,
};

use super::super::trackers::game_tick::*;

pub static FOOD_ITEMS: LazyLock<HashMap<Item, (i32, f32)>> = LazyLock::new(|| {
    HashMap::from([
        (Item::Apple, (4, 2.4)),
        (Item::BakedPotato, (5, 6.0)),
        (Item::Beef, (3, 1.8)),
        (Item::Beetroot, (1, 1.2)),
        (Item::BeetrootSoup, (6, 7.2)),
        (Item::Bread, (5, 6.0)),
        (Item::Carrot, (3, 3.6)),
        (Item::Chicken, (2, 1.2)),
        (Item::Cod, (2, 0.4)),
        (Item::CookedBeef, (8, 12.8)),
        (Item::CookedChicken, (6, 7.2)),
        (Item::CookedCod, (5, 6.0)),
        (Item::CookedMutton, (6, 9.6)),
        (Item::CookedPorkchop, (8, 12.8)),
        (Item::CookedRabbit, (5, 6.0)),
        (Item::CookedSalmon, (6, 9.6)),
        (Item::Cookie, (2, 0.4)),
        (Item::DriedKelp, (1, 0.6)),
        (Item::EnchantedGoldenApple, (4, 9.6)),
        (Item::GlowBerries, (2, 0.4)),
        (Item::GoldenApple, (4, 9.6)),
        (Item::GoldenCarrot, (6, 14.4)),
        (Item::HoneyBottle, (6, 1.2)),
        (Item::MelonSlice, (2, 1.2)),
        (Item::MushroomStew, (6, 7.2)),
        (Item::Mutton, (2, 1.2)),
        (Item::Porkchop, (3, 1.8)),
        (Item::Potato, (1, 0.6)),
        (Item::PumpkinPie, (8, 4.8)),
        (Item::Rabbit, (3, 1.8)),
        (Item::RabbitStew, (10, 12.0)),
        (Item::Salmon, (2, 0.4)),
        (Item::SweetBerries, (2, 0.4)),
        (Item::TropicalFish, (1, 0.2)),
    ])
});

pub struct AutoEatPlugin;


impl Plugin for AutoEatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            GameTick,
            Self::handle_auto_eat
                .after(GameTickPlugin::handle_game_ticks)
                .before(handle_outgoing_packets)
                .before(continue_mining_block)
                .before(InventorySet)
                .before(PhysicsSet),
        );
    }
}

type QuerySet<'a> = (
    Entity,
    &'a GameTicks,
    &'a Hunger,
    &'a Inventory,
    &'a LookDirection,
);

type QueryFilter = (With<Player>, With<LocalEntity>);

impl AutoEatPlugin {
    // Panic if slot > u16
    fn handle_auto_eat(
        mut query: Query<QuerySet, QueryFilter>,
        mut packet_events: EventWriter<SendPacketEvent>,
        mut container_click_events: EventWriter<ContainerClickEvent>,
    ) {
        for(entity, game_ticks, hunger, invetory, direction) in &mut query {
                if hunger.food >= 18 {
                    continue;
                }
                if !FOOD_ITEMS.contains_key(&invetory.held_item().kind()) {
                    let mut food_slots = Vec::new();
                    for slot in invetory.inventory_menu.player_slots_range() {
                        let Some(item) = invetory.inventory_menu.slot(slot) else {
                            continue;
                        };
                        if let Some((nutrition, saturation)) = FOOD_ITEMS.get(&item.kind()) {
                            food_slots.push((slot, *nutrition, *saturation));
                        }
                    }
                    food_slots.sort_by(|a, b| {
                        b.2.partial_cmp(&a.2)
                            .unwrap_or(Ordering::Equal)
                            .then_with(|| b.1.cmp(&a.1))
                    });
    
                    if let Some((slot, _, _)) = food_slots.first() {
                        container_click_events.send(ContainerClickEvent {
                            entity,
                            window_id: invetory.id,
                            operation: ClickOperation::Swap(SwapClick {
                                source_slot: u16::try_from(*slot).unwrap(),
                                target_slot: invetory.selected_hotbar_slot,
                            }),
                        });
                    }
                }
    
                let packet = ServerboundGamePacket::UseItem(ServerboundUseItem {
                    hand:     InteractionHand::MainHand,
                    pitch:    direction.x_rot,
                    yaw:      direction.y_rot,
                    sequence: 0,
                });
    
                packet_events.send(SendPacketEvent {
                    sent_by: entity,
                    packet,
                });
                
        }
    }
}
