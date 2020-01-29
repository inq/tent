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
                if class_names.is_empty() {
                    write!(f, "<{}", name)?;
                } else {
                    write!(f, "<{} class=\"{}\"", name, class_names.join(" "))?;
                }
                for (name, value) in properties {
                    write!(f, " {}=\"{}\"", name, value)?;
                }
                // TODO: Implement empty tag
                write!(f, ">")?;
                for content in contents.iter() {
                    content.fmt(f)?;
                }
                write!(f, "</{}>", name)
            }
            Content::Text(text) => write!(f, "{}", text),
        }
    }
}

impl Content {
    pub fn new_element(
        name: String,
        class_names: Vec<String>,
        properties: Vec<(String, String)>,
        contents: Vec<Content>,
    ) -> Self {
        Self::Element {
            name,
            class_names,
            properties,
            contents,
        }
    }

    pub fn new_text(text: String) -> Self {
        Self::Text(text)
    }
}
