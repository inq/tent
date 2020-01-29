use std::fmt;

#[derive(Debug)]
pub enum Content {
    Element {
        name: String,
        class_names: Vec<String>,
        properties: Vec<(String, String)>,
        contents: Vec<Content>,
    },
    Text(String),
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Content::Element {
                name,
                class_names,
                properties,
                contents,
            } => {
                write!(
                    f,
                    "tent::Content::Element {{ name: \"{}\".to_string(), ",
                    name
                )?;
                write!(f, "class_names: vec![")?;
                for class_name in class_names.iter() {
                    write!(f, "\"{}\".to_string(),", class_name)?;
                }
                write!(f, "], ")?;
                write!(f, "properties: vec![")?;
                for (name, value) in properties.iter() {
                    write!(f, "(\"{}\".to_string(),{}.to_string()),", name, value)?;
                }
                write!(f, "], ")?;
                write!(f, "contents: vec![")?;
                for content in contents.iter() {
                    write!(f, "{},", content)?;
                }
                write!(f, "] }}")
            }
            Content::Text(text) => write!(f, "tent::Content::Text({}.to_string())", text),
        }
    }
}
