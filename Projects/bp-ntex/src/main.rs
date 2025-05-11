mod utils;
mod driver;
mod browser;
mod handlers;

use ntex::web;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    println!("Starting WebDriver proxy server on 127.0.0.1:8080");
    
    web::HttpServer::new(|| {
        web::App::new()
            .service(handlers::index)
            .service(handlers::bp)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}