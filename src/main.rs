use std::error::Error;

use trivial_sql_builder::pgsql;

fn main() -> Result<(), Box<dyn Error>> {
    // println!("Hello, world!");
    //let my_str: &str = "xyzzy";
    println!("sql: {}", pgsql().
        s("SELECT ").
        // s(my_str).
        // XXX auto comma & auto space?
        text(&"x\nb'z".to_string()).c().
        int(-22).
        s(" FROM ").
        i(&"lr2".to_string()).dot().
        i(&"roles".to_string()).
        // i(&"ro\"les".to_string()).
        //s("WHERE x=").
        //s(my_str).
        //text(&"xyz".to_string()).
        //varchar(&"dog".to_string(), 4).
        //smallint(22i16).
        //int(-22).
        build()?);
        
    Ok(())
}
