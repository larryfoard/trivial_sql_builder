use anyhow::{Result, format_err};
use chrono::{NaiveDateTime};
use std::fmt::{Debug, Display, Write};
use once_cell::sync::Lazy;
use regex::Regex;
// TODO DB specific encoders

#[derive(Debug)]
pub struct SQL {
    // successfully built string
    value: String,
    // failure message
    failure: Option<String>,
}

#[warn(dead_code)]
impl SQL {
    pub fn new(capacity : usize) -> Self {
        Self {
            // avoid excessive realloc's
            value : String::with_capacity(capacity),
            failure: None,
        }
    }

    // PRIVATE
    fn write_is_safe_as_is<T: Display>(&mut self, value: T) {
        write!(&mut self.value, "{}", value).unwrap();
    }

    // escape identifier
    fn escape_identifier(&mut self, value: &str) -> () {
        let first = value.chars().next();

        let any_to_escape = value.
            chars().
            any(|c| 
                !(c.is_ascii_lowercase() || c.is_ascii_digit() || 
                  c == '_'));

        match (first, first.unwrap_or('X').is_ascii_digit() || any_to_escape)  {
            // invalid identifier
            (None, _) =>
                 self.failure = Some("empty identifier".into()),

            // need to quote
            (_, true) => {
                // needs escape and quoting
                self.value.push('"');
                
                value.
                    chars().
                    for_each(|c| {
                        match c {
                            // disallowed to avoid \0 attacks on C code
                            '\0' => 
                                self.failure = Some("string contains the null character".into()),

                            // escape "
                            '"' => self.value.push_str("\"\""),

                            '\x08' => self.value.push_str("\\b"),
                            '\x0C' => self.value.push_str("\\f"),
                            '\n'   => self.value.push_str("\\n"),
                            '\r'   => self.value.push_str("\\r"),
                            '\t'   => self.value.push_str("\\t"),

                            // otherwise pass through
                            c => self.value.push(c),
                        };
                    });
                
                self.value.push('"');
            },

            // no change needed
            (_, _) => self.value.push_str(value),
        };
    }

    // escape string
    fn escape_string(&mut self, value: &str) -> () {
        let any_to_escape = value.
            chars().
            any(|c|
                match c {
                    // special case we will reject
                    '\0' => true,
                    // simply escape
                    '\'' | '\x08' | '\x0C' | '\n' | '\r' | '\t' => true,
                    _ => false
                });
        
        if any_to_escape {
            self.value.push_str("E\'");
            value.
                chars().
                for_each(|c| {
                    match c {
                        '\0'   => {
                            self.failure = Some("Zero character in string".into());
                        },
                        '\\'   => self.value.push_str("\\\\"),
                        '\''   => self.value.push_str("\\'"),
                        '\x08' => self.value.push_str("\\b"),
                        '\x0C' => self.value.push_str("\\f"),
                        '\n'   => self.value.push_str("\\n"),
                        '\r'   => self.value.push_str("\\r"),
                        '\t'   => self.value.push_str("\\t"),
                        c      => self.value.push(c),
                    }
                });
            self.value.push('\'');
            
        } else {
            self.value.push('\'');
            self.value.push_str(value);
            self.value.push('\'');
        }
    }

    // add a failure
    pub fn fail(&mut self, failure: &str) {
        match &mut self.failure {
            Some(f) => {
                f.push_str(", ");
                f.push_str(failure);
            },
            None => {
                self.failure = Some(failure.to_string());
            },
        }

        self.value.push_str("<<<FAILURE ");
    }


    // append another SQL object
    pub fn push_sql(mut self, value: &SQL) -> Self {
        if let Some(failure) = &value.failure {
            // propagate failures
            self.fail(failure);
        }

        self.value.push_str(&value.value);

        self
    }

    // join an iterator yielding SQL objects
    pub fn append_join(mut self, delimit: &SQL, values: &Vec<SQL>) -> Self {
        let mut need_delimit = false;

        for sql in values {
            if need_delimit {
                self = self.push_sql(delimit);
            }

            self = self.push_sql(&sql);

            need_delimit = true;
        }

        self
    }

    pub fn format(mut self, values: Vec<(&str, SQL)>) -> Self {
        static RE: Lazy<Regex> = Lazy::new(||
            Regex::new(r"(\\\{|\{[^}]+}|[^{]+)").unwrap());

        let old_value = self.value;
        // hopefully minimize reallocs
        self.value = String::with_capacity(4000);

        RE.find_iter(old_value.as_str()).
            for_each(|v| {
                let v = v.as_str();
                match (v.starts_with('{'), v.ends_with('}'), v == "\\{") {
                    // replace with variable
                    (true, true, false) => {
                        let v = &v[1..v.len()-1];
                        
                        match values.iter().
                            find(|(name, _)| *v == **name) {
                            
                            None => {
                                self.fail(format!("missing variable: {{v}}").as_str());
                                self.value.push_str(format!("{v} <- VARIABLE NOT FOUND   ").as_str());
                            },

                            Some((_, sql)) => self.value.push_str(&sql.value),
                        }
                    }
                    
                    // a lone escaped '{'
                    (false, false, true) => self.value.push('{'),
                    
                    // everything else is a literal
                    _ => self.value.push_str(v),
                }
            });

        self
    }

    pub fn build(self) -> Result<String> {
        if let Some(failure) = self.failure {
            Err(format_err!("{}", failure))?;
        }
        
        Ok(self.value)
    }

    pub fn build_borrowed(&self) -> Result<&String> {
        if let Some(failure) = &self.failure {
            Err(format_err!("{}", failure))?;
        }
        
        Ok(&self.value)
    }
    
    
    // static encoder methods
    pub fn text(value: &str) -> SQL {
        let mut result = SQL::new(value.len() * 2);
        result.escape_string(value);
        result
    }

    pub fn varchar(value: &String, max : usize) -> SQL {
        let size = value.chars().count();
        let mut result = SQL::new(size * 2);
        if size > max {
            result.fail(format!("varchar too long: {} vs {}", size, max).as_str());
        } else {
            result.escape_string(value);
        }
        result
    }    

    // SQL text
    pub fn sql(value: &str) -> SQL {
        let mut result = SQL::new(value.len() * 2);
        result.write_is_safe_as_is(value);
        result
    }

    // numbers
    pub fn smallint(value: i16) -> SQL {
        let mut result = SQL::new(7);
        result.write_is_safe_as_is(value);
        result
    }

    pub fn int(value: i32) -> SQL {
        let mut result = SQL::new(12);
        result.write_is_safe_as_is(value);
        result
    }

    pub fn integer(value: i32) -> SQL {
        let mut result = SQL::new(12);
        result.write_is_safe_as_is(value);
        result
    }

    pub fn bigint(value: i64) -> SQL {
        let mut result = SQL::new(24);
        result.write_is_safe_as_is(value);
        result
    }

    pub fn real(value: f32) -> SQL {
        let mut result = SQL::new(100);
        result.write_is_safe_as_is(value);
        result
    }

    pub fn double(value: f64) -> SQL {
        let mut result = SQL::new(100);
        result.write_is_safe_as_is(value);
        result
    }

    pub fn boolean(value: bool) -> SQL {
        let mut result = SQL::new(7);
        result.write_is_safe_as_is(
            if value {
                "TRUE"
            } else {
               "FALSE"
            }
        );
        result
    }

    // identifier
    pub fn identifier(value: &str) -> SQL {
        let mut result = SQL::new(value.len() * 2);
        result.escape_identifier(value);
        result
    }

    // naive date time
    pub fn naive_date_time(value: &NaiveDateTime) -> SQL {
        let mut result = SQL::new(30);
        result.escape_string(&value.to_string());
        result
    }

    pub fn join(delimit: &SQL, values: &Vec<SQL>) -> Self {
        // TODO figure size in advance?
        let result = SQL::new(100);

        result.append_join(delimit, values)
    }

    // build a valid WHERE clause empty values uses the
    // on_empty boolean
    pub fn clause(delimit: &SQL, values: &Vec<SQL>, on_empty: bool) -> Self {
        if values.is_empty() {
            Self::boolean(on_empty)
        } else {
            let mut result = SQL::new(100);
            result.write_is_safe_as_is("(\n");
            result = result.append_join(delimit, values);
            result.write_is_safe_as_is("\n)\n");
            result
        }
    }

    pub fn and(values: &Vec<SQL>, on_empty: bool) -> Self {
        Self::clause(&SQL::sql(" AND\n"), values, on_empty)
    }

    pub fn or(values: &Vec<SQL>, on_empty: bool) -> Self {
        Self::clause(&SQL::sql(" OR\n"), values, on_empty)
    }

    // build an IN clause that can be empty, on_empty used when
    // empty.
    // expr IN (values)
    pub fn in_vec(expr: &SQL, values: &Vec<SQL>, on_empty: bool) -> Self {
        if values.is_empty() {
            Self::boolean(on_empty)
        } else {
            let mut result = SQL::new(100);
            result = result.push_sql(expr);
            result.write_is_safe_as_is(" IN (");
            result = result.append_join(&SQL::sql(", "), values);
            result.write_is_safe_as_is(")\n");
            result
        }
    }
}

