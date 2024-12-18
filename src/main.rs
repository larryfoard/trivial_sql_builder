use std::error::Error;

use trivial_sql_builder::pgsql;

fn main() -> Result<(), Box<dyn Error>> {
    // println!("Hello, world!");
    let my_str: &str = "xyzzy";
    println!("sql: {}", pgsql().
        // s("SELECT * FROM ").
        s(my_str).
        i(&"lr2.roles".to_string()).
        //s("WHERE x=").
        s(my_str).
        text(&"xyz".to_string()).
        varchar(&"dog".to_string(), 4).
        smallint(22i16).
        int(-22).
        ii(&["cat".to_string(), "dog".to_string()]).
        build()?);
        
    Ok(())
}
