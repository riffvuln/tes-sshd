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
    CsvWriter::new(&mut fl).include_header(true).with_separator(b',').finish(&mut def1)?;

    // Read the CSV from the file
    let mut fl = File::open("data.csv")?;
    let df_fl = CsvReadOptions::default()
        .with_has_header(true)
        .with_parse_options(CsvParseOptions::default().with_try_parse_dates(true))
        .try_into_reader_with_file_path(Some("data.csv".into()))?
        .finish()?;

    // Expression and Contexts
    // Select: Select columns and apply expressions
    let select_df = df_fl.clone()
    .lazy()
    .select([
        col("name"),
        col("Birthday").dt().year().alias("Birth Year"),
        (col("weight")/col("height").pow(2)).alias("bmi")
    ]).collect()?;
    
    // Print the DataFrame
    println!("Original DataFrame\n{}", df_fl);
    println!("Select DataFrame\n{}", select_df);
    Ok(())
}