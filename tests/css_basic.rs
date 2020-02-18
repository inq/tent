#![feature(proc_macro_hygiene)]

#[test]
fn test_simple() -> Result<(), failure::Error> {
    assert_eq!(
        tent::css!(
            body
                fontFamily: "sans-serif"
            .notice
                width: "400px"
                height: "300px"
                margin: "auto"
                fontSize: "2em"
                textAlign: "center"
                .head
                    fontSize: "4em"
                .content
                    fontSize: "0.5em"
                    lineHeight: "1.5em"
        )
        .to_string(),
        vec![
            "body {font-family: sans-serif;}",
            ".notice .head {font-size: 4em;}",
            ".notice .content {font-size: 0.5em;line-height: 1.5em;}",
            ".notice {width: 400px;height: 300px;margin: auto;font-size: 2em;text-align: center;}",
        ]
        .join(""),
    );
    Ok(())
}
