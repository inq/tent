#[test]
fn test_simple() -> Result<(), failure::Error> {
    assert_eq!(
        tent::css!(
            r#"
            body
                fontFamily: "sans-serif"
            #idTest
                width: "100px"
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
        "#
        )
        .to_string(),
        vec![
            "body {font-family: sans-serif;}",
            "#idTest {width: 100px;}",
            ".notice .head {font-size: 4em;}",
            ".notice .content {font-size: 0.5em;line-height: 1.5em;}",
            ".notice {width: 400px;height: 300px;margin: auto;font-size: 2em;text-align: center;}",
        ]
        .join(""),
    );
    Ok(())
}

#[test]
fn test_font() -> Result<(), failure::Error> {
    assert_eq!(
        tent::css!(
            r#"
            @fontFace
                fontFamily: "myfont"
                src: "url('/assets/font.woff') format('woff')"
        "#
        )
        .to_string(),
        vec![
            "@font-face {",
            "font-family: myfont;",
            "src: url(\'/assets/font.woff\') format(\'woff\');",
            "}",
        ]
        .join(""),
    );
    Ok(())
}
