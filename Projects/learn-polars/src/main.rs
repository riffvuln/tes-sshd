use polars::prelude::*;

fn main() {
    let dataf = df!(
        "NAME" => ["Alice", "Bob", "Charlie"],
        "AGE" => [25, 30, 35],
        "CITY" => ["New York", "Los Angeles", "Chicago"],
        "SALARY" => [70000.0, 80000.0, 90000.0],
        "IS_EMPLOYED" => [true, false, true],
        "JOIN_DATE" => [
            "2020-01-01",
            "2019-05-15",
            "2021-07-20"
        ]
    ).unwrap();


    println!("DataFrame:\n{}", dataf);
}
