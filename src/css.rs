use std::fmt;

#[derive(Debug)]
pub enum Item {
    Node { name: String, children: Vec<Item> },
    Declaration(String, String),
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::Node { name, children } => {
                write!(f, "{} {{", name)?;
                for child in children.iter() {
                    write!(f, "{}", child)?;
                }
                write!(f, "}}")
            }
            Item::Declaration(key, value) => write!(f, "{}: {};", key, value),
        }
    }
}

#[derive(Debug)]
pub struct Content {
    pub items: Vec<Item>,
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for item in self.items.iter() {
            write!(f, "{}", item)?;
        }
        Ok(())
    }
}
