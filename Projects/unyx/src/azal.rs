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
    let state = State::default();

        ClientBuilder::new()
            .set_handler(handle_event(tx_log))
            .start(account, SERVER_ADDRESS)
            .await
            .unwrap();
        Ok(())
    }
    
    fn handle_event(tx_log: std::sync::mpsc::Sender<ConsoleType>) -> impl Handler<State> {
        move |_bot: Client, event: Event, _state: State| {
            let tx_log = tx_log.clone();
            async move {
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
            }
        }
        .start(account, SERVER_ADDRESS)
        .await
        .unwrap();
    Ok(())
}