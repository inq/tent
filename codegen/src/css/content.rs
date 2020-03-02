use std::fmt;

#[derive(Debug)]
pub enum Item {
    Node { name: String, children: Vec<Item> },
    Declaration(String, String),
}

impl Item {
    fn flatten(self, namespace: Option<String>) -> (Option<Item>, Vec<Item>) {
        match self {
            Item::Node { name, children } => {
                let my_name = if let Some(namespace) = namespace {
                    format!("{} {}", namespace, name)
                } else {
                    name
                };

                let mut declarations = vec![];
                let mut nodes = vec![];
                for child in children {
                    let (declaration, mut child_nodes) = child.flatten(Some(my_name.clone()));
                    if let Some(declaration) = declaration {
                        declarations.push(declaration);
                    }
                    nodes.append(&mut child_nodes);
                }
                nodes.push(Item::Node {
                    name: my_name,
                    children: declarations,
                });
                (None, nodes)
            }
            declaration => (Some(declaration), vec![]),
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::Node { name, children } => {
                write!(f, "tent::CssItem::Node {{ name: \"{}\".to_string(), ", name)?;
                write!(f, "children: vec![")?;
                for child in children.iter() {
                    write!(f, "{},", child)?;
                }
                write!(f, "] }}")
            }
            Item::Declaration(key, value) => write!(
                f,
                "tent::CssItem::Declaration(\"{}\".to_string(), {}.to_string())",
                key, value
            ),
        }
    }
}

#[derive(Debug)]
pub struct Content {
    pub items: Vec<Item>,
}

impl Content {
    pub fn flatten(self) -> Self {
        let mut res = vec![];
        for item in self.items {
            let (declaration, mut nodes) = item.flatten(None);
            assert!(declaration.is_none());
            res.append(&mut nodes);
        }
        Self { items: res }
    }
}

impl fmt::Display for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tent::CssContent {{ items: vec![",)?;
        for item in self.items.iter() {
            write!(f, "{},", item)?;
        }
        write!(f, "] }}")
    }
}
