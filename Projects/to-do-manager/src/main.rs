use serde::{Serialize, Deserialize};

// Modules
mod io;

// Re-exports
use io::IOEngine;

#[derive(Serialize, Deserialize)]
struct DataHandler {
    io_engine: IOEngine<'static>,
}

impl DataHandler {
    pub fn new(file_path: &'static str) -> Self {
        Self { io_engine: IOEngine::new(file_path) }
    }

    pub fn read_data(&self) -> Option<String> {
        self.io_engine.read()
    }

    pub fn write_data(&self, content: &str) -> bool {
        self.io_engine.write(content)
    }
}

pub (crate) fn main() -> Result<(), Box<dyn std::error::Error>> {
    let io_engine = IOEngine::new("data.json");

    
    let data = io_engine.read_data();
    println!("Data read: {:?}", data);

    let write_success = io_engine.write_data("New content");
    println!("Write successful: {}", write_success);

    Ok(())
}
