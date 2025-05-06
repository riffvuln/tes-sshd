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
    let block_state = azalea::registry::
    let blocks = bot.world().read().find_blocks(bot.position(), block_states)
    Ok(())
}