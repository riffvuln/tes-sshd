use color_eyre::Result;


mod rats;
mod azal;

use azal::ConsoleType;

const SERVER_ADDRESS: &'static str = "kalwi.id";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let (tx_log, rx_log) = std::sync::mpsc::channel::<ConsoleType>();
    std::thread::spawn(move || ratatui_term(rx_log));
    std::thread::spawn(|| deadlock_detector());
    azal::start_azalea(SERVER_ADDRESS, tx_log).await?;

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

fn ratatui_term(rx: std::sync::mpsc::Receiver<ConsoleType>) -> Result<()> {
    let terminal = ratatui::init();
    let mut rat_app = rats::RatApp::new();

    std::thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(ConsoleType::Botlog(msg)) => {
                    let mut bot_log = rat_app.bot_log.lock();
                    bot_log.push(msg);
                }
                Ok(ConsoleType::ServerMsg(msg)) => {
                    let mut server_msgs = rat_app.server_msgs.lock();
                    server_msgs.push(msg);
                }
                Err(_) => break,
            }
        }
    });
    
    
    let app_result = rat_app.run(terminal);
    ratatui::restore();
    app_result
}

