use crate::harness::functional_testing::selectors_tests::{get_test_cases, TestCase};
use crate::harness::functional_testing::FunctionalTestFixture;
use crate::harness::Output;
use cool_thing::{ContentType, ElementContentHandlers, HtmlRewriterBuilder};

pub struct SelectorMatchingTests;

impl FunctionalTestFixture<TestCase> for SelectorMatchingTests {
    fn test_cases() -> Vec<TestCase> {
        get_test_cases("selector_matching")
    }

    fn run(test: &TestCase) {
        let encoding = test.input.encoding().unwrap();
        let mut output = Output::new(encoding);
        let mut first_text_chunk_expected = true;

        {
            let mut builder = HtmlRewriterBuilder::default();

            builder
                .on(
                    &test.selector,
                    ElementContentHandlers::default()
                        .element(|el| {
                            el.before(
                                &format!("<!--[ELEMENT('{}')]-->", test.selector),
                                ContentType::Html,
                            );
                            el.after(
                                &format!("<!--[/ELEMENT('{}')]-->", test.selector),
                                ContentType::Html,
                            );
                        })
                        .comments(|c| {
                            c.before(
                                &format!("<!--[COMMENT('{}')]-->", test.selector),
                                ContentType::Html,
                            );
                            c.after(
                                &format!("<!--[/COMMENT('{}')]-->", test.selector),
                                ContentType::Html,
                            );
                        })
                        .text(|t| {
                            if first_text_chunk_expected {
                                t.before(
                                    &format!("<!--[TEXT('{}')]-->", test.selector),
                                    ContentType::Html,
                                );

                                first_text_chunk_expected = false;
                            }

                            if t.last_in_text_node() {
                                t.after(
                                    &format!("<!--[/TEXT('{}')]-->", test.selector),
                                    ContentType::Html,
                                );

                                first_text_chunk_expected = true;
                            }
                        }),
                )
                .unwrap();

            let mut rewriter = builder
                .build(encoding.name(), |c: &[u8]| output.push(c))
                .unwrap();

            for chunk in test.input.chunks() {
                rewriter.write(chunk).unwrap();
            }

            rewriter.end().unwrap();
        }

        let actual: String = output.into();

        assert_eq!(actual, test.expected);
    }
}

functional_test_fixture!(SelectorMatchingTests);
