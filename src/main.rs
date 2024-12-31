use std::error::Error;
use chrono::{NaiveDate, NaiveTime, NaiveDateTime};

use trivial_sql_builder::SQL;

fn main() -> Result<(), Box<dyn Error>> {
    // println!("Hello, world!");
    //let my_str: &str = "xyzzy";
    // let a = [1, 2, 3 ,4];
    // let b = a.iter().map(|&v| SQL().int(v));
    
    let date = NaiveDate::from_ymd_opt(2021, 12, 25).expect(":(");
    let time = NaiveTime::from_hms_opt(10, 30, 0).expect(":(");
    let datetime = NaiveDateTime::new(date, time);
    
    println!("sql: {}", 
        SQL::sql("SELECT {a}, {b}, {d}, {e} FROM {c}").
        format(vec![
            ("a", SQL::text(&"x\nb'z".to_string())),
            ("b", SQL::int(-22)),
            ("c", SQL::identifier(&"lr2".to_string())),
            ("d", SQL::double(2.23)),
            ("e", SQL::naive_date_time(&datetime)),
        ]).
        build()?);

    println!("sql: {:?}", 
        SQL::sql("SELECT {a}, {b}, {d} FROM {c}").
        format(vec![
            ("a", SQL::text(&"x\nb'z".to_string())),
            ("c", SQL::identifier(&"lr2".to_string())),
            ("d", SQL::double(2.23))
        ]).
        build());
        
    Ok(())
}
