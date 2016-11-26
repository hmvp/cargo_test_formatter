use nom::{IResult, digit, alphanumeric, eol, not_line_ending};

use std::str;
use std::str::FromStr;

use {Test, TestResult, Failure, TestModule};


/// 1
named!(number<u32>,
  map_res!(
    map_res!(
      digit,
      str::from_utf8
    ),
    FromStr::from_str
  )
);

///
/// tests::test_test_case
named!(test_name<&str>, map_res!(
          is_not!(" "),
          str::from_utf8
        )
);

/// ok|FAILED|ignored
named!(test_result<TestResult>,
    do_parse!(
        result: map_res!(
          alphanumeric,
          str::from_utf8
        ) >>
        ( if result == "ok" {
            TestResult::Ok
        } else if result== "ignored" {
            TestResult::Ignored
        } else {
            TestResult::Failed
        })
    )
);

named!(test_start<u32>, terminated!(
    delimited!(
        tag!("running "),
        number,
        alt_complete!(
            tag!(" tests") | tag!(" test")
        )
    ),
    eol
));

named!(test_end<(TestResult,u32, u32, u32, u32)>, do_parse!(
    tag!("test result: ") >>
    result: test_result   >>
    tag!(". ")            >>
    passed: number        >>
    tag!(" passed; ")     >>
    failed: number        >>
    tag!(" failed; ")     >>
    ignored: number       >>
    tag!(" ignored; ")    >>
    measured: number      >>
    tag!(" measured")     >>
    eol                   >>
    (result, passed, failed, ignored, measured)
));

///
/// test tests::test_test_case ... ok\r\n
named!(test_function<Test>, terminated!(
    do_parse!(
        tag!("test ")       >>
        name: test_name     >>
        tag!(" ... ")       >>
        result: test_result >>
        (Test(name, result))
    ),
    eol
));

named!(failure<Failure>, do_parse!(
    eol >>
    name:   terminated!(delimited!(tag!("---- "), test_name , tag!(" stdout ----")), eol) >>
    tag!("\t") >>
    stdout: map_res!(take_until!("thread"), str::from_utf8) >>
    info:   terminated!(map_res!(not_line_ending, str::from_utf8), eol) >>
    opt!(terminated!(
            tag!("note: Run with `RUST_BACKTRACE=1` for a backtrace."), eol
    )) >>
    stack: opt!(delimited!(
            terminated!(tag!("stack backtrace:"), eol),
            map_res!(take_until!("\n\n"), str::from_utf8),
            eol
    )) >>
    (Failure(name, stdout, info, stack.unwrap_or("")))
));

named!(failures<Vec<Failure> >, do_parse!(
    terminated!(tag!("failures:"), eol) >>
    failure_data: many1!(failure) >>
    eol >>
    eol >>
    terminated!(tag!("failures:"), eol) >>
    many1!(delimited!(tag!("    "), not_line_ending, eol)) >>
    eol >>
    (failure_data)
));

named!(test_module<TestModule>, do_parse!(
    test_start >>
    tests: terminated!(many0!(test_function), eol) >>
    failures: opt!(failures) >>
    end: test_end >>
    (TestModule(end.0, tests, failures.unwrap_or(vec![]), end.1, end.2, end.3,end.4))
));

named!(test_suite<Vec<TestModule> >, terminated!(
    many1!(delimited!(eol, test_module,opt!(eol))),
    eof!()
));


pub fn parse(string: &[u8]) -> Result<Vec<TestModule>, String> {
    match test_suite(string) {
        IResult::Done(b"", result) => Ok(result),
        r => Err(format!("parse failure: {:?}", r).to_string()),
    }
}

#[cfg(test)]
mod tests {

    use nom::IResult;

    use {Test, TestResult, Failure, TestModule};
    use super::{number, test_name, test_result, test_start, test_end, test_function, failure,
                failures, test_module, test_suite};


    #[test]
    fn test_number() {
        assert_eq!(number(b"0"), IResult::Done(&b""[..], 0));
        assert_eq!(number(b"1"), IResult::Done(&b""[..], 1));
        assert_eq!(number(b"99999"), IResult::Done(&b""[..], 99999));
    }
    #[test]
    fn test_test_name() {
        assert_eq!(test_name(b"tests::test_test_case"),
            IResult::Done(&b""[..], "tests::test_test_case"));
        assert_eq!(test_name(b"test_test_case"),
            IResult::Done(&b""[..], "test_test_case"));
        assert_eq!(test_name(b"tests::test_test_CASE::xxx"),
            IResult::Done(&b""[..], "tests::test_test_CASE::xxx"));
    }
    #[test]
    fn test_test_result() {
        assert_eq!(test_result(b"ok"),
            IResult::Done(&b""[..], TestResult::Ok));
        assert_eq!(test_result(b"FAILED"),
            IResult::Done(&b""[..], TestResult::Failed));
    }

    #[test]
    fn test_test_start() {
        assert_eq!(test_start(b"running 1 test\r\n"),
      IResult::Done(&b""[..], 1));
        assert_eq!(test_start(b"running 0 tests\r\n"),
      IResult::Done(&b""[..], 0));
    }

    #[test]
    fn test_test_end() {
        assert_eq!(test_end(b"test result: ok. 60 passed; 2 failed; 3 ignored; 0 measured\r\n"),
      IResult::Done(&b""[..], (TestResult::Ok,60,2,3,0)));
        assert_eq!(test_end(b"test result: ok. 10 passed; 2 failed; 3 ignored; 4 measured\r\n"),
      IResult::Done(&b""[..], (TestResult::Ok,10,2,3,4)));
        assert_eq!(test_end(b"test result: FAILED. 60 passed; 2 failed; 3 ignored; 0 measured\r\n"),
      IResult::Done(&b""[..], (TestResult::Failed,60,2,3,0)));
    }

    #[test]
    fn test_test_function() {
        assert_eq!(test_function(b"test tests::test_test_case ... ok\r\n"),
      IResult::Done(&b""[..], Test("tests::test_test_case", TestResult::Ok)));
    }

    #[test]
    fn test_test_failure() {
        assert_eq!(failure(include_bytes!("../tests/test_failure.txt")),
      IResult::Done(&b"\x0A"[..], Failure("tests::test_failing2",
       "Again!!\n",
       "thread 'tests::test_failing2' panicked at 'assertion failed: \
        `(left == right)` (left: `no`, right: `yes`)', src/main.rs:243",
        "")));
    }

    #[test]
    fn test_test_failures() {
        assert_eq!(failures(include_bytes!("../tests/test_failures.txt")),
      IResult::Done(&b""[..], vec![
          Failure("tests::test_failing",
           "Oh noes!!\n",
           "thread 'tests::test_failing' panicked at 'assertion failed: \
            false', src/main.rs:250", ""),
          Failure("tests::test_failing2",
           "Again!!\n",
           "thread 'tests::test_failing2' panicked at 'assertion failed: \
            `(left == right)` (left: `no`, right: `yes`)', src/main.rs:255", "")
      ]));
    }

    #[test]
    fn test_test_module() {
        assert_eq!(test_module(include_bytes!("../tests/test_module.txt")),
      IResult::Done(&b""[..], TestModule(
              TestResult::Ok,vec![Test("tests::test_test_case",
                  TestResult::Ok),
              Test("tests::test_test_function",
                  TestResult::Ok)], vec![],1,2,3,4)));

        assert_eq!(test_module(include_bytes!("../tests/test_module2.txt")),
      IResult::Done(&b""[..],
          TestModule(
              TestResult::Ok,
              vec![
                  Test("tests::test_test_case",TestResult::Ok),
                  Test("tests::test_test_function",TestResult::Ok)
              ],
              vec![
                  Failure("tests::test_failing",
                      "Oh noes!!\n", "thread \'tests::test_failing\' panicked at \
                      \'assertion failed: false\', src/main.rs:250", ""),
                  Failure("tests::test_failing2",
                      "Again!!\n", "thread \'tests::test_failing2\' panicked at \
                      \'assertion failed: `(left == right)` (left: `no`, right: `yes`)\', src/main.rs:255", "")
              ],1,2,3,4)));
    }

    #[test]
    fn test_test_suite() {
        assert_eq!(test_suite(include_bytes!("../tests/test_suite.txt")),
      IResult::Done(&b""[..], vec![
          TestModule(TestResult::Ok,vec![
              Test("tests::test_test_case",TestResult::Ok),
              Test("tests::test_test_function",TestResult::Ok)
          ],
          vec![
              Failure("tests::test_failing",
              "Oh noes!!\n", "thread \'tests::test_failing\' panicked at \'assertion failed: \
              false\', src/main.rs:250", ""),
              Failure("tests::test_failing2",
              "Again!!\n", "thread \'tests::test_failing2\' panicked at \'assertion failed: \
              `(left == right)` (left: `no`, right: `yes`)\', src/main.rs:255", "")
          ], 1,2,3,4)
      ]));

    }
}
