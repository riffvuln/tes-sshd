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

