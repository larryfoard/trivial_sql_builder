use std::error::Error;

use trivial_sql_builder::SQL;

fn main() -> Result<(), Box<dyn Error>> {
    // println!("Hello, world!");
    //let my_str: &str = "xyzzy";
    // let a = [1, 2, 3 ,4];
    // let b = a.iter().map(|&v| SQL().int(v));
    
    println!("sql: {}", 
        SQL::sql("SELECT {a}, {b}, {d} FROM {c}").
        format(vec![
            ("a", SQL::text(&"x\nb'z".to_string())),
            ("b", SQL::int(-22)),
            ("c", SQL::identifier(&"lr2".to_string())),
            ("d", SQL::double(2.23))
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
