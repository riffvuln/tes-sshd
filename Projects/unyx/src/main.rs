use color_eyre::Result;


mod rats;
mod azal;

use azal::ConsoleType;
use azal::CommandType;

const SERVER_ADDRESS: &'static str = "mp.earthmc.net";

#[tokio::main]
async fn main() -> Result<()> {
    unsafe {
        std::env::set_var("RUST_LOG", "warn,azalea_client::plugins::packet::game::events=off");
    }
    color_eyre::install()?;
    let (tx_log, rx_log) = std::sync::mpsc::channel::<ConsoleType>();
    let (tx_input, rx_input) = std::sync::mpsc::channel::<CommandType>();
    std::thread::spawn(move || ratatui_term(rx_log, tx_input));
    std::thread::spawn(|| deadlock_detector());
    azal::start_azalea(SERVER_ADDRESS, tx_log, rx_input).await?;

    Ok(())
}

fn deadlock_detector() {
    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));
        let deadlocks = parking_lot::deadlock::check_deadlock();
        if deadlocks.is_empty() {
            continue;
        }

        println!("{} deadlocks detected", deadlocks.len());
        for (i, threads) in deadlocks.iter().enumerate() {
            println!("Deadlock #{i}");
            for t in threads {
                println!("Thread Id {:#?}", t.thread_id());
                println!("{:#?}", t.backtrace());
            }
        }
    }
}

fn ratatui_term(
    rx_log: std::sync::mpsc::Receiver<ConsoleType>, 
    tx_input: std::sync::mpsc::Sender<CommandType>,
) -> Result<()> {
    let terminal = ratatui::init();
    let mut rat_app = rats::RatApp::new();
    
    // Clone the Arc fields before moving them
    let bot_log_clone = rat_app.bot_log.clone();
    let server_msgs_clone = rat_app.server_msgs.clone();

    std::thread::spawn(move || {
        loop {
            match rx_log.recv() {
                Ok(ConsoleType::Botlog(msg)) => {
                    let mut bot_log = bot_log_clone.lock().unwrap();
                    bot_log.push(msg);
                }
                Ok(ConsoleType::ServerMsg(msg)) => {
                    let mut server_msgs = server_msgs_clone.lock().unwrap();
                    server_msgs.push(msg);
                }
                Err(_) => break,
            }
        }
    });
    
    
    let app_result = rat_app.run(terminal, tx_input);
    ratatui::restore();
    app_result
}

