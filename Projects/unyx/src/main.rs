use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");
    panic!("This is a panic message");
    Ok(())
}
