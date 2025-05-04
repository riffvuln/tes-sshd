use color_eyre::Result;
use parking_lot::Mutex;
use azalea::prelude::*;

use color_eyre::Result;

#[derive(Default, Clone, Component)]
pub struct State {}

pub async fn start_azalea(
    address: &str,
) -> Result<()> {
    let account = Account::offline("ItzBtzz");

    Ok(())
}