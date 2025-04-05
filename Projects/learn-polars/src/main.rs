use polars::prelude::*;
use chrono::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let def1: DataFrame = df!(
        "name" => ["Alice", "Bob", "Charlie"],
        "Birthday" => [
            NaiveDate::from_ymd_opt(2009, 7, 12).unwrap(),
            NaiveDate::from_ymd_opt(2008, 5, 23).unwrap(),
            NaiveDate::from_ymd_opt(2007, 3, 15).unwrap(),
        ],
        "weight" => [60.0, 70.0, 80.0],
        "height" => [1.65, 1.75, 1.85],
    )?;

    println!("{}", def1);
    Ok(())
}