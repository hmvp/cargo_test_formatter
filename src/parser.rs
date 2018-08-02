use nom::{IResult, digit, alphanumeric, eol, not_line_ending};

use std::str::FromStr;

use {Test, TestResult, Failure, TestModule};


/// 1
named!(number<&str, u32>,
  map_res!(
    digit,
    FromStr::from_str
  )
);

/// ok|FAILED|ignored
named!(test_result<&str, TestResult>,
    do_parse!(
        result: alphanumeric >>
        ( if result == "ok" {
            TestResult::Ok
        } else if result== "ignored" {
            TestResult::Ignored
        } else {
            TestResult::Failed
        })
    )
);

named!(test_start<&str, u32>, terminated!(
    delimited!(
        tag_s!("running "),
        number,
        alt_complete!(
            tag_s!(" tests") | tag_s!(" test")
        )
    ),
    eol
));

named!(test_end<&str, (TestResult,u32, u32, u32, u32, u32)>, do_parse!(
    tag_s!("test result: ") >>
    result: test_result   >>
    tag_s!(". ")            >>
    passed: number        >>
    tag_s!(" passed; ")     >>
    failed: number        >>
    tag_s!(" failed; ")     >>
    ignored: number       >>
    tag_s!(" ignored; ")    >>
    measured: number      >>
    tag_s!(" measured")    >>
    filtered: opt!(delimited!(
        tag_s!("; "),
        number,
        tag_s!(" filtered out")
    )) >>
    eol                   >>
    (result, passed, failed, ignored, measured, filtered.unwrap_or(0))
));

/// Test line
///  
/// ## Normal test
/// ```
/// test tests::test_test_case ... ok\r\n
/// ```
/// 
/// # Doc test
/// ```
/// test src/hexfile.rs - hexfile::MBHexFile::new (line 102) ... ok
/// ```
named!(test_function<&str, Test>, do_parse!(
    tag_s!("test ")       >>
    name: take_until_and_consume_s!(" ... ") >>
    result: test_result >>
    eol >>
    (Test(name, result))
));

named!(failure<&str, Failure>, do_parse!(
    name:   delimited!(tag_s!("---- "), take_until_and_consume_s!(" stdout ----"), eol) >>
    stdout: ws!(take_until_s!("thread")) >>
    info: terminated!(not_line_ending, eol) >>
    info_left_right: opt!(
        tuple!(tag_s!("  left: "), not_line_ending, tag_s!("\n right: "), take_until_and_consume_s!("\n"))) >>
    opt!(
        tuple!(tag_s!("note: "), not_line_ending, eol)) >>
    stack: opt!(delimited!(
            terminated!(tag_s!("stack backtrace:"), eol),
            take_until_s!("\n\n"),
            eol
    )) >>
    eol >>
    (Failure(name, stdout, if let Some(lr) = info_left_right {vec![info, "\n", lr.0, lr.1, lr.2, lr.3]} else {vec![info]}, stack.unwrap_or("")))
));

named!(failures<&str, Vec<Failure> >, do_parse!(
    terminated!(tag_s!("failures:"), eol) >>
    eol >>
    failure_data: many1!(failure) >>
    eol >>
    terminated!(tag_s!("failures:"), eol) >>
    many1!(delimited!(tag_s!("    "), not_line_ending, eol)) >>
    eol >>
    (failure_data)
));

named!(test_module<&str, TestModule>, do_parse!(
    test_start >>
    tests: terminated!(many0!(test_function), eol) >>
    failures: opt!(failures) >>
    end: test_end >>
    (TestModule(end.0, tests, failures.unwrap_or(vec![]), end.1, end.2, end.3,end.4, end.5))
));

named!(test_suite<&str, Vec<TestModule> >, terminated!(
    many1!(delimited!(eol, test_module,opt!(eol))),
    eof!()
));


pub fn parse(string: &str) -> Result<Vec<TestModule>, String> {
    let result: IResult<&str, _> = test_suite(string);
    match result {
        IResult::Done("", result) => Ok(result),
        r => Err(format!("parse failure: {:?}", r).to_string()),
    }
}

#[cfg(test)]
mod tests {

    use nom::IResult;

    use {Test, TestResult, Failure, TestModule};
    use super::{number, test_result, test_start, test_end, test_function, failure,
                failures, test_module, test_suite};


    #[test]
    fn test_number() {
        assert_eq!(number("0"), IResult::Done("", 0));
        assert_eq!(number("1"), IResult::Done("", 1));
        assert_eq!(number("99999"), IResult::Done("", 99999));
    }

    #[test]
    fn test_test_result() {
        assert_eq!(test_result("ok"),
            IResult::Done("", TestResult::Ok));
        assert_eq!(test_result("FAILED"),
            IResult::Done("", TestResult::Failed));
    }

    #[test]
    fn test_test_start() {
        assert_eq!(test_start("running 1 test\r\n"),
      IResult::Done("", 1));
        assert_eq!(test_start("running 0 tests\r\n"),
      IResult::Done("", 0));
    }

    #[test]
    fn test_test_end() {
        assert_eq!(test_end("test result: ok. 60 passed; 2 failed; 3 ignored; 0 measured; 0 filtered out\r\n"),
      IResult::Done("", (TestResult::Ok,60,2,3,0,0)));
        assert_eq!(test_end("test result: ok. 10 passed; 2 failed; 3 ignored; 4 measured; 0 filtered out\r\n"),
      IResult::Done("", (TestResult::Ok,10,2,3,4,0)));
        assert_eq!(test_end("test result: FAILED. 60 passed; 2 failed; 3 ignored; 0 measured; 1 filtered out\r\n"),
      IResult::Done("", (TestResult::Failed,60,2,3,0,1)));
    }

    #[test]
    fn test_test_function() {
        assert_eq!(test_function("test tests::test_test_case ... ok\r\n"),
      IResult::Done("", Test("tests::test_test_case", TestResult::Ok)));
    }

    #[test]
    fn test_test_failure() {
        assert_eq!(failure(include_str!("../tests/test_failure.txt")),
      IResult::Done("", Failure("tests::test_failing2",
       "Again!!\n",
       vec!["thread 'tests::test_failing2' panicked at 'assertion failed: `(left == right)`", "\n",
        "  left: ", "`\"no\"`,", "\n right: ", "`\"yes\"`', src\\main.rs:100:9"],
        "")));
    }

    #[test]
    fn test_test_failures() {
        assert_eq!(failures(include_str!("../tests/test_failures.txt")),
      IResult::Done("", vec![
          Failure("tests::test_failing",
           "Oh noes!!\n",
           vec!["thread 'tests::test_failing' panicked at 'assertion failed: false', src\\main.rs:93:12"], ""),
          Failure("tests::test_failing2",
           "Again!!\n",
           vec!["thread 'tests::test_failing2' panicked at 'assertion failed: `(left == right)`", "\n",
            "  left: ", "`\"no\"`,", "\n right: ", "`\"yes\"`', src\\main.rs:100:9"], "")
      ]));
    }

    #[test]
    fn test_test_module() {
        assert_eq!(test_module(include_str!("../tests/test_module.txt")),
      IResult::Done("", TestModule(
            TestResult::Ok,vec![
                Test("tests::test_test_case", TestResult::Ok),
                Test("test_test_case", TestResult::Ok),
                Test("tests::test_test_CASE::xxx", TestResult::Ok),
                Test("src/hexfile.rs - hexfile::MBHexFile::new (line 102)", TestResult::Ok),
                Test("tests::test_test_function", TestResult::Ok)
            ], vec![],1,2,3,4,5)));

        assert_eq!(test_module(include_str!("../tests/test_module2.txt")),
      IResult::Done("",
          TestModule(
              TestResult::Ok,
              vec![
                  Test("tests::test_test_case",TestResult::Ok),
                  Test("tests::test_test_function",TestResult::Ok)
              ],
              vec![
                  Failure("tests::test_failing",
                      "Oh noes!!\n", vec!["thread 'tests::test_failing' panicked at 'assertion failed: false', src\\main.rs:93:12"], ""),
                  Failure("tests::test_failing2",
                      "Again!!\n", vec!["thread 'tests::test_failing2' panicked at 'assertion failed: `(left == right)`", "\n",
                        "  left: ", "`\"no\"`,", "\n right: ", "`\"yes\"`', src\\main.rs:100:9"], "")
              ],1,2,3,4,5)));
    }

    #[test]
    fn test_empty_module() {
        assert_eq!(test_module(include_str!("../tests/test_empty_module.txt")),
      IResult::Done("", TestModule(
              TestResult::Ok,vec![], vec![],0,0,0,0,0)));
    }

    #[test]
    fn test_test_suite() {
        assert_eq!(test_suite(include_str!("../tests/test_suite.txt")),
      IResult::Done("", vec![
          TestModule(TestResult::Ok,vec![
              Test("tests::test_test_case",TestResult::Ok),
              Test("tests::test_test_function",TestResult::Ok)
          ],
          vec![
              Failure("tests::test_failing",
              "Oh noes!!\n", vec!["thread 'tests::test_failing' panicked at 'assertion failed: false', src\\main.rs:93:12"], ""),
              Failure("tests::test_failing2",
              "Again!!\n", vec!["thread 'tests::test_failing2' panicked at 'assertion failed: `(left == right)`", "\n",
                "  left: ", "`\"no\"`,", "\n right: ", "`\"yes\"`', src\\main.rs:100:9"], "")
          ], 1,2,3,4,5),
          TestModule(TestResult::Ok,vec![
              Test("src/hexfile.rs - hexfile::MBHexFile::new (line 102)", TestResult::Ok),
          ],vec![], 1,0,0,0,0)
      ]));

    }

    #[test]
    fn test_test_suite_with_stack_backtrace() {
        assert_eq!(test_suite(include_str!("../tests/test_suite_with_stack_backtrace.txt")),
      IResult::Done("", vec![
          TestModule(TestResult::Ok,vec![
              Test("tests::test_test_case",TestResult::Ok),
              Test("tests::test_test_function",TestResult::Ok)
          ],
          vec![
              Failure("tests::test_failing",
              "Oh noes!!\n", vec!["thread 'tests::test_failing' panicked at 'assertion failed: false', src\\main.rs:93:12"],
              "   0: std::sys::windows::backtrace::unwind_backtrace
             at C:\\projects\\rust\\src\\libstd\\sys\\windows\\backtrace\\mod.rs:65
   1: std::sys_common::backtrace::_print
             at C:\\projects\\rust\\src\\libstd\\sys_common\\backtrace.rs:71"),
              Failure("tests::test_failing2",
              "Again!!\n", vec!["thread 'tests::test_failing2' panicked at 'assertion failed: `(left == right)`", "\n",
                "  left: ", "`\"no\"`,", "\n right: ", "`\"yes\"`', src\\main.rs:100:9"],
                "   0: std::sys::windows::backtrace::unwind_backtrace
             at C:\\projects\\rust\\src\\libstd\\sys\\windows\\backtrace\\mod.rs:65
   1: std::sys_common::backtrace::_print
             at C:\\projects\\rust\\src\\libstd\\sys_common\\backtrace.rs:71")
          ], 1,2,3,4,5),
          TestModule(TestResult::Ok,vec![
              Test("src/hexfile.rs - hexfile::MBHexFile::new (line 102)", TestResult::Ok),
          ],vec![], 1,0,0,0,0)
      ]));

    }
}
