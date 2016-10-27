#[macro_use]
extern crate nom;
extern crate xml;

use std::io;
use std::fs::File;

mod parser;

use parser::parse;



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
pub struct TestModule<'a>(TestResult, Vec<Test<'a>>, Vec<Failure<'a>>, u32, u32, u32, u32);



fn parse_data<'a, T>(reader: &mut T) -> Vec<u8>
    where T: io::Read + 'a
{
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer).expect("Empty input");

    buffer
}

fn format(data: Vec<TestModule>) -> xml::Element {
    let mut output = xml::Element::new("testsuites".into(), None, vec![]);

    for module in data {
        let attr = vec![
            ("failures".into(), None, format!("{}",module.4).into()),
            ("skip".into(), None, format!("{}",module.6).into()),
            ("tests".into(), None, format!("{}",module.1.len()).into()),
        ];
        output.tag(xml::Element::new("testsuite".into(), None, attr));

        for test in module.1 {
            let (basename, classname) = test.0
                .rfind("::")
                .map(|i| (test.0[2 + i..].into(), test.0[..i].replace("::", ".")))
                .unwrap_or((test.0, "::".into()));

            let attr = vec![
                       ("name".into(), None, basename.into()),
                       ("classname".into(), None, classname.into()),
            ];

            let test_xml = output.tag(xml::Element::new("testcase".into(), None, attr));

            if test.1 == TestResult::Ignored {
                test_xml.tag(xml::Element::new("skipped".into(), None, vec![]));
            } else if test.1 == TestResult::Failed {
                for failure in &module.2 {
                    if failure.0 == test.0 {
                        test_xml.tag(xml::Element::new("failure".into(),
                                                   None,
                                                   vec![
                                    ("message".into(), None, failure.2.into()),
                                    ]))
                            .cdata(failure.3.into());
                        test_xml.tag(xml::Element::new("system-out".into(), None, vec![]))
                            .text(failure.1.into());

                    }
                }
            }
        }
    }

    output
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

    let data = parse(&string);

    if let Ok(data) = data {
        let output = format(data);
        println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", output);
    } else {
        println!("Something went wrong{:#?}", data);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_ok() {
        println!("Oh noes!!");
        assert!(true);
    }
    #[test]
    #[ignore]
    fn test_failing2() {
        println!("Again!!");
        assert_eq!("no", "yes");
    }

    //    #[test]
    //    fn test_failing() {
    //        println!("Oh noes!!");
    //        assert!(false);
    //    }
    //    #[test]
    //    fn test_failing2() {
    //        println!("Again!!");
    //        assert_eq!("no", "yes");
    //    }
}
