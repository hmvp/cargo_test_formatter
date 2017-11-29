#[macro_use]
extern crate nom;
extern crate xml;

use std::io;
use std::fs::File;
use std::io::Write;

mod parser;
mod junit;



#[derive(Debug, PartialEq)]
pub enum TestResult {
    Ok,
    Ignored,
    Failed,
}
#[derive(Debug, PartialEq)]
pub struct Failure<'a>(&'a str, &'a str, &'a str, &'a str);
#[derive(Debug, PartialEq)]
pub struct Test<'a>(&'a str, TestResult);
#[derive(Debug, PartialEq)]
pub struct TestModule<'a>(
    TestResult,
    Vec<Test<'a>>,
    Vec<Failure<'a>>,
    u32,
    u32,
    u32,
    u32,
    u32,
);



fn parse_data<'a, T>(reader: &mut T) -> String
where
    T: io::Read + 'a,
{
    let mut string = String::new();

    reader.read_to_string(&mut string).expect("Empty input");

    string
}


///
/// # Example
/// ```
/// assert!(true);
/// ```
fn main() {
    let filename: Option<&str> = None;
    let stdin = io::stdin();

    let string = if let Some(filename) = filename {
        if let Ok(mut file) = File::open(filename) {
            parse_data(&mut file)
        } else {
            parse_data(&mut stdin.lock())
        }
    } else {
        parse_data(&mut stdin.lock())
    };

    let data = parser::parse(&string);

    if let Ok(data) = data {
        let output = junit::format(data);
        junit::print(output);
    } else {
        std::io::stderr()
            .write_fmt(format_args!("Error during parsing{:#?}", data))
            .expect("Error during printing of error");
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_ok() {
        println!("Oh noes!!");
        assert!(true);
    }

    //    #[test]
    //    fn test_failing() {
    //        println!("Oh noes!!");
    //        assert!(false);
    //    }

    #[test]
    #[ignore]
    fn test_failing2() {
        println!("Again!!");
        assert_eq!("no", "yes");
    }
}
