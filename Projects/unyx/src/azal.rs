use color_eyre::Result;
use parking_lot::Mutex;
use azalea::prelude::*;

#[derive(Default, Clone, Component)]
pub struct State {}

pub enum ConsoleType {
    Botlog(String),
    ServerMsg(String),
}

pub async fn start_azalea(
    address: &str,
    tx_log: Option<tokio::sync::mpsc::Sender<ConsoleType>>,
) -> Result<()> {
    let account = Account::offline("ItzBtzz");
    
    Ok(())
}