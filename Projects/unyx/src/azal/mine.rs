use azalea::{
    ecs::prelude::*,
    prelude::*,
};
use color_eyre::eyre::Ok;

use super::State;

pub fn mine_by_block_id(bot: Client, state: State, block_id: i32, quantity: i32) -> color_eyre::Result<()> {
    if block_id > 1104 {
        return Ok(());
    }
    let mut mined = 0;
    let block_state = unsafe {
        azalea::registry::Block::from_u32_unchecked(block_id as u32)
    };
    let block_states = azalea::blocks::BlockStates::from(block_state);
    let blocks = bot.world().read().find_blocks(bot.position(), &block_states);

    Ok(())
}