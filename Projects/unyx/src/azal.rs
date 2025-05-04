use color_eyre::Result;
use parking_lot::Mutex;
use azalea::prelude::*;

use crate::SERVER_ADDRESS;

#[derive(Clone, Component, Default)]
pub struct State {}

pub enum ConsoleType {
    Botlog(String),
    ServerMsg(String),
}

pub async fn start_azalea(
    address: &str,
    tx_log: std::sync::mpsc::Sender<ConsoleType>,
) -> Result<()> {
    let account = Account::offline("ItzBtzz");

    ClientBuilder::new()
        .set_handler(handle)
        .start(account, SERVER_ADDRESS)
        .await
        .unwrap();
    Ok(())
}

async fn handle(bot: Client, event: Event, state: State) -> color_eyre::Result<()> {

    Ok(())
}