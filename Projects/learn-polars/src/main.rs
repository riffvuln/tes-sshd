use polars::prelude::*;
use chrono::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let def1: DataFrame = df!(
        "name" => ["Alice", "Bob", "Charlie"],
        "Birthday" => [
            NaiveDate::from_ymd_opt(2009, 7, 12)?,
            NaiveDate::from_ymd_opt(2008, 5, 23)?,
            NaiveDate::from_ymd_opt(2007, 3, 15)?
        ]
    )
    Ok(())
}