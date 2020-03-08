#![feature(proc_macro_hygiene)]

#[test]
fn test_simple() -> Result<(), failure::Error> {
    assert_eq!(
        tent::html!(
            html
                body
                    span.hello "HELLO!"
                    .hello {"Inner Text"}
        )
        .to_string(),
        vec![
            "<html><body>",
            "<span class=\"hello\">HELLO!</span>",
            "<div class=\"hello\">Inner Text</div>",
            "</body></html>"
        ]
        .join("")
    );
    Ok(())
}

#[test]
fn test_text_node() -> Result<(), failure::Error> {
    assert_eq!(
        tent::html!(
            html
                body
                    span.hello
                        "HELLO!"
                        div "Hi"
                    .hello {"Inner Text"}
        )
        .to_string(),
        vec![
            "<html><body>",
            "<span class=\"hello\">HELLO!<div>Hi</div></span>",
            "<div class=\"hello\">Inner Text</div>",
            "</body></html>"
        ]
        .join("")
    );
    Ok(())
}

#[test]
fn test_svg_node() -> Result<(), failure::Error> {
    assert_eq!(
        tent::html!(
            html
                body
                    span.hello dataTest="test-data" "HELLO!"
                    svg version="1.1" viewBox="0 0 1 1"
                        path d=""
        )
        .to_string(),
        vec![
            "<html><body>",
            "<span class=\"hello\" data-test=\"test-data\">HELLO!</span>",
            "<svg version=\"1.1\" viewBox=\"0 0 1 1\">",
            "<path d=\"\"></path>",
            "</svg>",
            "</body></html>"
        ]
        .join("")
    );
    Ok(())
}
