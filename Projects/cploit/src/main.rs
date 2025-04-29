use cploit::{lazyconfig, extractdb, extractcpanel};

fn print_banner() {
    println!(" ██████╗  █████╗ ");
    println!("██╔═████╗██╔══██╗   -/ By: ItzYuuRz - Zero - Farell Haksor");
    println!("██║██╔██║╚██████║   -/ cpanel cracker tools 2025");
    println!("████╔╝██║ ╚═══██║   -/ best tools for get cpanel from laravel and joomla");
    println!("╚██████╔╝ █████╔╝██╗");
    println!(" ╚═════╝  ╚════╝ ╚═╝  - created 23 April 2024 by Caterscam\n");
    println!("1. scan .env config from laravel & SFTP");
    println!("2. extractor Database");
    println!("3. extrac Database for cpanel crack");
    println!("4. cpanel crack from Database\n");
}

fn main() {
    print_banner();
    println!("--------------------------------------------");
    let mut input = String::new();
    println!("Choose an option:");
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let choice: u32 = input.trim().parse().expect("Please enter a number");
    match choice {
        1 => {
            lazyconfig::run_lazy_config().expect("goblok opsi 1 error tuh");
        }
        2 => {
            let _ = tokio::runtime::Runtime::new()
            .expect("Failed to create Tokio runtime")
            .block_on(extractdb::run());
        }
        3 => {
            let _ = extractcpanel::extract_cpanel();
        }
        // 4 => {
        //     let _ = tokio::runtime::Runtime::new()
        //         .expect("Failed to create Tokio runtime")
        //         .block_on(cpanelcrack::crack_cpanel());
        // }
        _ => {
            println!("Invalid choice, please try again.");
        }
    }
}
