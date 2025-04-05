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
        Person::new("Alice", 30, "New York", 70000.0, true, NaiveDate::from_ymd(2020, 1, 1)),
        Person::new("Bob", 25, "Los Angeles", 50000.0, false, NaiveDate::from_ymd(2021, 6, 15)),
        Person::new("Charlie", 35, "Chicago", 80000.0, true, NaiveDate::from_ymd(2019, 3, 10)),   
    ).unwrap();

    dataf.iter().for_each(|s| {
        println!("Series: {:?}", s);
    });
    println!("DataFrame:\n{}", dataf);
}
