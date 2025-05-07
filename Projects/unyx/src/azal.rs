// Modules
mod killaura;
mod mine;
mod modules;
mod trackers;
pub mod prelude;

// Re-exports
use color_eyre::Result;
use modules::ModulesPluginGroup;
use parking_lot::Mutex;
use azalea::{pathfinder::goals::{BlockPosGoal, XZGoal}, prelude::*, BlockPos};
use trackers::TrackersGroup;
use std::sync::mpsc::{Receiver, Sender};
use once_cell::sync::Lazy;
use killaura::tick_mob_killaura;
use mine::mine_by_block_id;
pub use prelude::*;

#[derive(Default, Clone, Component)]
pub struct State {
    pub mob_killaura: bool,
    pub is_on_task: bool
}

impl State {
    pub fn new() -> Self {
        Self { mob_killaura: true, is_on_task: false }
    }
}

pub enum ConsoleType {
    Botlog(String),
    ServerMsg(String),
}

#[derive(Clone)]
pub enum CommandType {
    Chat(String),
    Goto(String),
    Mobkillaura(bool),
    Mine(String),
}

// Global variable to store the sender
static TX_LOG: Lazy<Mutex<Option<Sender<ConsoleType>>>> = Lazy::new(|| Mutex::new(None));
static RX_INPUT: Lazy<Mutex<Option<Receiver<CommandType>>>> = Lazy::new(|| Mutex::new(None));

async fn handle(bot: Client, event: Event, mut state: State) -> color_eyre::Result<()> {
    match event {
        Event::Login => {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            bot.chat("/login rifaiGG123");
            bot.chat("/register rifaiGG123 rifaiGG123");
        }
        Event::Chat(m) => {
            let message = m.message().to_ansi();
            
            // Send to the channel if available?
            // why i use ? because that's an option!!! :333
            if let Some(tx) = &*TX_LOG.lock() {
                let _ = tx.send(ConsoleType::ServerMsg(message));
                // let _ = tx.send(ConsoleType::Botlog("GOT MESSAGE".to_string())); // fucking idiot
            }
        }
        Event::Tick => {
            if state.mob_killaura {
                tick_mob_killaura(bot.clone(), state.clone())?;
            }
        }
        _ => {}
    }
    
    // try debug: Use try_recv() instead of recv() to make it non-blocking
    // Oh that's work nicee :333
    let rx_input = RX_INPUT.lock();
    if let Some(rx) = &*rx_input {
        match rx.try_recv() {
            Ok(CommandType::Chat(msg)) => {
                bot.chat(&msg);
            }
            Ok(CommandType::Goto(msg)) => {
                let msg = msg.split_whitespace().collect::<Vec<_>>();
                if msg.len() == 3 {
                    let x = msg[0].parse::<i32>().unwrap();
                    let y = msg[1].parse::<i32>().unwrap();
                    let z = msg[2].parse::<i32>().unwrap();
                    bot.goto(BlockPosGoal(BlockPos::new(x, y, z)));
                } else if msg.len() == 2 {
                    let x = msg[0].parse::<i32>().unwrap();
                    let z = msg[1].parse::<i32>().unwrap();
                    bot.goto(XZGoal { x: x, z: z});
                } else {
                    bot.chat("Invalid coordinates");
                }
            }
            Ok(CommandType::Mobkillaura(enabled)) => {
                state.mob_killaura = enabled;
            }
            Ok(CommandType::Mine(msg)) => {
                let msg = msg.split_whitespace().collect::<Vec<_>>();
                let block_id = msg[0].parse::<i32>().unwrap();
                let quantity = msg[1].parse::<i32>().unwrap();
                mine_by_block_id(bot.clone(), state.clone(), block_id, quantity)?;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // No message available, that's fine :3
            }
            Err(_) => {
                // Channel is disconnected :/
            }
        }
    }
    Ok(())
}

fn init_handler(tx: Sender<ConsoleType>, rx: Receiver<CommandType>) {
    *TX_LOG.lock() = Some(tx);
    *RX_INPUT.lock() = Some(rx);
}

pub async fn start_azalea(
    address: &str,
    tx_log: std::sync::mpsc::Sender<ConsoleType>,
    rx_input: std::sync::mpsc::Receiver<CommandType>,
) -> Result<()> {
    let account = Account::offline("ItzBtzz");
    
    // Initialize the global sender
    init_handler(tx_log, rx_input);

    ClientBuilder::new()
        .add_plugins(TrackersGroup)
        .add_plugins(ModulesPluginGroup)
        .set_handler(handle)
        .set_state(State::new())
        .start(account, address)
        .await
        .unwrap();
    Ok(())
}