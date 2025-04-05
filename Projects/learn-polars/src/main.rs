use polars::prelude::*;
use chrono::prelude::*;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut def1: DataFrame = df!(
        "name" => ["Alice", "Bob", "Charlie"],
        "Birthday" => [
            NaiveDate::from_ymd_opt(2009, 7, 12).unwrap(),
            NaiveDate::from_ymd_opt(2008, 5, 23).unwrap(),
            NaiveDate::from_ymd_opt(2007, 3, 15).unwrap(),
        ],
        "weight" => [60.0, 70.0, 80.0],
        "height" => [1.65, 1.75, 1.85],
    )?;

    // Save the DataFrame to a CSV file
    let mut fl = File::create("data.csv")?;
    CsvWriter::new(&mut fl).include_header(true).with_separator([b","]).finish(&mut def1)?;

    println!("{}", def1);
    Ok(())
}