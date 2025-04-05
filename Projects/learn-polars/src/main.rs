use chrono::prelude::*;
use polars::prelude::*;

struct Person<'a> {
    name: &'a str,
    age: u32,
    city: &'a str,
    salary: f64,
    is_employed: bool,
    join_date: NaiveDate,
}

impl<'a> Person<'a> {
    fn new(name: &'a str, age: u32, city: &'a str, salary: f64, is_employed: bool, join_date: NaiveDate) -> Self {
        Person {
            name,
            age,
            city,
            salary,
            is_employed,
            join_date,
        }
    }
}

fn main() {
    let dataf = df!(
        
    ).unwrap();

    dataf.iter().for_each(|s| {
        println!("Series: {:?}", s);
    });
    println!("DataFrame:\n{}", dataf);
}
