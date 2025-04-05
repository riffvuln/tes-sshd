use std::path::Iter;

use chrono::prelude::*;
use polars::prelude::*;

struct ListDataFrame {
    frames: Vec<DataFrame>,
}

impl ListDataFrame {
    fn new() -> Self {
        ListDataFrame { frames: Vec::new() }
    }

    fn add_frame(&mut self, frame: DataFrame) {
        self.frames.push(frame);
    }

    fn get_frame(&self, index: usize) -> Option<&DataFrame> {
        self.frames.get(index)
    }
}

impl Iterator for ListDataFrame {
    type Item = DataFrame;

    fn next(&mut self) -> Option<Self::Item> {
        self.frames.pop()
    }
}

fn main() {
    let mut list_df = ListDataFrame::new();
    let mut df1 = df!(
        "a" => &[1, 2, 3],
        "b" => &[4, 5, 6],
    )
    .unwrap();
    let mut df2 = df!(
        "c" => &[7, 8, 9],
        "d" => &[10, 11, 12],
    )
    .unwrap();
    let mut df3 = df!(
        "e" => &[13, 14, 15],
        "f" => &[16, 17, 18],
    )
    .unwrap();
    list_df.add_frame(df1);
    list_df.add_frame(df2);
    list_df.add_frame(df3);

    for frame in list_df {
        println!("DataFrame:\n{}", frame);
    }
}
