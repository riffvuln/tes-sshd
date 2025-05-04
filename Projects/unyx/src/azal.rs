use color_eyre::Result;
use parking_lot::Mutex;
use azalea::prelude::*;

use crate::SERVER_ADDRESS;

#[derive(Default, Clone, Component)]
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

    let handle = move |bot: Client, event: Event, state: State| -> color_eyre::Result<()> {
        match event {
            Event::Chat(m) => {
                tx_log
                    .send(ConsoleType::ServerMsg(format!(
                        "{}",
                        m.message().to_ansi()
                    )))
                    .unwrap();
            }
            _ => {}
        }
    
        Ok(())
    };

    ClientBuilder::new()
        .set_handler(handle)
        .start(account, address)
        .await
        .unwrap();
    Ok(())
}