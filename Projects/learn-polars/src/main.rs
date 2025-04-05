use chrono::prelude::*;
use polars::prelude::*;

fn main() {
    let dataf = df!(
        "NAME" => ["Alice", "Bob", "Charlie"],
        "AGE" => [25, 30, 35] as [u32; 3],
        "CITY" => ["New York", "Los Angeles", "Chicago"] as [&'static str; 3],
        "SALARY" => [70000.0, 80000.0, 90000.0],
        "IS_EMPLOYED" => [true, false, true],
        "JOIN_DATE" => [
            NaiveDate::from_ymd_opt(2020, 2, 12).unwrap(),
            NaiveDate::from_ymd_opt(2019, 5, 23).unwrap(),
            NaiveDate::from_ymd_opt(2021, 8, 30).unwrap()
        ]
    ).unwrap();
    
    println!("DataFrame:\n{}", dataf);
}
