use std::error::Error;
use std::fmt::{Display, Write};
// TODO DB specific encoders

pub struct PGSQL {
    // successfully built string
    value: String,
    // failure message
    failure: Option<String>,
}

#[warn(dead_code)]
impl PGSQL {
    pub fn new() -> Self {
        Self {
            // avoid excessive realloc's
            value : String::with_capacity(4096),
            failure: None,
        }
    }

    // SQL text
    pub fn s(mut self, value: &str) -> Self {
        self.value += "s: ";
        self.value += value;
        self.value += "\n";
        self
    }

    // Encoded SQL types
    pub fn text(mut self, value: &String) -> Self {
        self.value += "l: ";
        self.value += &value;
        self.value += "\n";
        self
    }

    pub fn varchar(mut self, value: &String, max : usize) -> Self {
        self.value += "v: ";
        let size = value.chars().count();
        if size > max {
            self.failure = Some(format!("varchar too long: {} vs {}", size, max));
        } else {
            // XXX encode
            self.value += &value;
        }
        self.value += "\n";
        self
    }
    
    // PRIVATE
    fn safe_as_is<T: Display>(&mut self, value: T) {
        // TODO skip {} formatting here?
        self.value += "safe_as_is: ";
        write!(&mut self.value, "{}", value).unwrap();
        self.value += "\n";
    }
    
    // numbers
    pub fn smallint(mut self, value: i16) -> Self {
        self.safe_as_is(value);
        self
    }

    pub fn int(mut self, value: i32) -> Self {
        self.safe_as_is(value);
        self
    }

    pub fn integer(mut self, value: i32) -> Self {
        self.safe_as_is(value);
        self
    }

    pub fn bigint(mut self, value: i64) -> Self {
        self.safe_as_is(value);
        self
    }

    pub fn real(mut self, value: f32) -> Self {
        self.safe_as_is(value);
        self
    }

    pub fn double(mut self, value: f64) -> Self {
        self.safe_as_is(value);
        self
    }
    
    // TODO arrays for IN


    pub fn i(mut self, value: &String) -> Self {
        self.value += "i: ";
        self.value += &value;
        self.value += "\n";
        self
    }

    // TODO table with two inputs

    pub fn ii(mut self, value: &[String]) -> Self {
        self.value += "i: ";
        self.value += &value.len().to_string();
        self.value += "\n";
        self
    }

    pub fn build(&self) -> Result<&String, Box<dyn Error>> {
        if let Some(failure) = &self.failure {
            Err(failure.clone())?;
        }
        
        Ok(&self.value)
    }
}

pub fn pgsql() -> PGSQL {
    PGSQL::new()
}

