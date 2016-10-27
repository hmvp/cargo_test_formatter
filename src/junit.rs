use xml;

use TestModule;
use TestResult;

pub fn format(data: Vec<TestModule>) -> xml::Element {
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
                .map_or((test.0, "::".into()),
                        |i| (test.0[2 + i..].into(), test.0[..i].replace("::", ".")));

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

pub fn print(output: xml::Element) {
    println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}", output);
}
