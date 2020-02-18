#![feature(proc_macro_hygiene)]

#[test]
fn test_simple() -> Result<(), failure::Error> {
    assert_eq!(
        tent::html!(
            html
                body
                    span.hello "HELLO!"
                    .hello {"Inner Text"}
        ).to_string(),
        "<html><body><span class=\"hello\">HELLO!</span><div class=\"hello\">Inner Text</div></body></html>".to_string()
    );
    Ok(())
}
