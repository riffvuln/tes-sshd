use azalea::{
    ecs::prelude::*, pathfinder::goals::BlockPosGoal, prelude::*
};
use color_eyre::eyre::Ok;
use std::io::Write;

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
    let world = bot.world();
    let readed_world = world.read();
    let blocks = readed_world.find_blocks(bot.position(), &block_states);
    let mut f = std::fs::File::create("blocks.txt")?;
    for block in blocks {
        let pos = (block.x, block.y, block.z);
        writeln!(f, "{pos:?}").unwrap();
    }
    for block in blocks {
        if mined >= quantity {
            break;
        }
        bot.goto(BlockPosGoal(block));
        // bot.start_mining(block);
        mined += 1;
    }
    // println!("Mined {mined} blocks of {block_id}");
    Ok(())
}