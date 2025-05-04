use color_eyre::Result;
use parking_lot::Mutex;
use azalea::prelude::*;


mod rats;

const SERVER_ADDRESS: &'static str = "kalwi.id";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    std::thread::spawn(ratatui_term);
    std::thread::spawn(deadlock_detector);

    let account = Acc

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

fn ratatui_term() -> Result<()> {
    let terminal = ratatui::init();
    let mut rat_app = rats::RatApp::new();
    
    let bot_log_clone = rat_app.bot_log.clone();
    
    std::thread::spawn(move ||{
        std::thread::sleep(std::time::Duration::from_secs(2));
        let mut bot_log = bot_log_clone.lock().unwrap();
        bot_log.push("Hello from the bot!".to_string());
    });
    
    let app_result = rat_app.run(terminal);
    ratatui::restore();
    app_result
}

