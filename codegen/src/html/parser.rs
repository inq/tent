use std::str::FromStr;

use crate::html::Content;
use proc_macro::{TokenStream, TokenTree};

#[derive(Debug)]
pub enum Node {
    Ident(String),
    Punct(char),
    Literal(String),
    Group(String),
}

#[derive(Default)]
pub struct LineBuilder {
    level: Option<usize>,
    nodes: Vec<Node>,
}

impl LineBuilder {
    fn new(level: usize) -> Self {
        Self {
            level: Some(level),
            ..Default::default()
        }
    }

    // TODO: Replace Option with Result
    fn build(self) -> Option<Line> {
        Some(Line {
            level: self.level?,
            nodes: self.nodes,
        })
    }

    fn put(&mut self, token: TokenTree) {
        let node = match token {
            TokenTree::Ident(ident) => Node::Ident(ident.to_string()),
            TokenTree::Punct(punct) => Node::Punct(punct.as_char()),
            TokenTree::Literal(literal) => Node::Literal(literal.to_string()),
            TokenTree::Group(group) => Node::Group(group.to_string()),
        };
        self.nodes.push(node);
    }
}

#[derive(Debug)]
pub struct Line {
    level: usize,
    nodes: Vec<Node>,
}

impl Line {
    fn property_name_to_dashed(property_name: &str) -> String {
        if property_name == "viewBox" {
            property_name.to_string()
        } else {
            crate::util::camelcase_to_dashed(property_name)
        }
    }

    pub fn process(self) -> Option<BuilderNode> {
        #[derive(Debug)]
        enum State {
            StandBy,
            HasIdent,
            HasPropertyName(String),
            NeedPropertyValue(String),
            NeedClassName,
            Done(BuilderNode),
        }
        let mut state = State::StandBy;
        let mut tag = None;
        let mut contents = vec![];
        let mut class_names = vec![];
        let mut properties: Vec<(String, String)> = vec![];

        for node in self.nodes.into_iter() {
            match (&state, node) {
                (State::StandBy, Node::Ident(ident)) => {
                    tag = Some(ident);
                    state = State::HasIdent;
                }
                (State::StandBy, Node::Literal(ref literal)) => {
                    state = State::Done(BuilderNode::Text {
                        level: self.level,
                        text: literal.to_string(),
                    });
                }
                (State::StandBy, Node::Group(ref group)) => {
                    state = State::Done(BuilderNode::Text {
                        level: self.level,
                        text: group.to_string(),
                    });
                }
                (State::StandBy, Node::Punct('.')) => {
                    tag = Some(String::from("div"));
                    state = State::NeedClassName;
                }
                (State::HasIdent, Node::Punct('.')) => {
                    state = State::NeedClassName;
                }
                (State::HasIdent, Node::Ident(ident)) => {
                    // Receive property name
                    state = State::HasPropertyName(Self::property_name_to_dashed(&ident));
                }
                (State::HasPropertyName(name), Node::Punct('=')) => {
                    state = State::NeedPropertyValue(name.to_string());
                }
                (State::NeedPropertyValue(name), Node::Literal(ref literal)) => {
                    properties.push((name.to_string(), literal.to_string()));
                    state = State::HasIdent;
                }
                (State::NeedPropertyValue(name), Node::Group(ref group)) => {
                    properties.push((name.to_string(), group.to_string()));
                    state = State::HasIdent;
                }
                (State::HasIdent, Node::Literal(literal)) => {
                    // TODO: Implement more
                    contents.push(literal);
                }
                (State::HasIdent, Node::Group(group)) => {
                    // TODO: Implement
                    contents.push(group);
                }
                (State::NeedClassName, Node::Ident(ident)) => {
                    class_names.push(ident);
                    state = State::HasIdent;
                }
                (state, node) => {
                    panic!("{:?}, {:?}", state, node);
                }
            }
        }
        if let State::Done(res) = state {
            Some(res)
        } else if contents.is_empty() {
            Some(BuilderNode::Tag {
                level: self.level,
                tag: tag?,
                class_names,
                properties,
                children: vec![],
            })
        } else {
            Some(BuilderNode::InlineTag {
                level: self.level,
                tag: tag?,
                class_names,
                properties,
                contents,
            })
        }
    }
}

#[derive(Debug)]
pub struct Parser {
    lines: Vec<Line>,
}

#[derive(Debug)]
pub enum BuilderNode {
    InlineTag {
        level: usize,
        tag: String,
        class_names: Vec<String>,
        properties: Vec<(String, String)>,
        contents: Vec<String>, // TODO: Use enum
    },
    Tag {
        level: usize,
        tag: String,
        class_names: Vec<String>,
        properties: Vec<(String, String)>,
        children: Vec<Content>,
    },
    Text {
        level: usize,
        text: String,
    },
}

impl BuilderNode {
    fn level(&self) -> usize {
        match self {
            Self::InlineTag { level, .. } => *level,
            Self::Tag { level, .. } => *level,
            Self::Text { level, .. } => *level,
        }
    }

    pub fn into_element(self) -> Content {
        match self {
            Self::InlineTag {
                tag,
                class_names,
                properties,
                contents,
                ..
            } => Content::Element {
                name: tag,
                class_names,
                properties,
                contents: contents.into_iter().map(Content::Text).collect(),
            },
            Self::Tag {
                tag,
                class_names,
                properties,
                children,
                ..
            } => Content::Element {
                name: tag,
                class_names,
                properties,
                contents: children,
            },
            Self::Text { text, .. } => Content::Text(text),
        }
    }

    pub fn set_children(&mut self, new_children: Vec<Content>) {
        if let Self::Tag {
            ref mut children, ..
        } = self
        {
            *children = new_children;
        } else {
            panic!("Unreachable");
        }
    }
}

impl Parser {
    pub fn from_str(input: &str) -> Option<Self> {
        let lines = input
            .lines()
            .map(|line| {
                let mut tokens = TokenStream::from_str(line).unwrap().into_iter();
                let mut line_builder =
                    LineBuilder::new(line.find(|c: char| !c.is_whitespace()).unwrap_or_default());
                while let Some(token) = tokens.next() {
                    line_builder.put(token);
                }
                line_builder.build().unwrap()
            })
            .collect();

        Some(Self { lines })
    }

    fn clean_stack(stack: &mut Vec<BuilderNode>) {
        let leaf_level = stack.last().unwrap().level();
        let mut leaves = vec![];
        while stack.last().unwrap().level() == leaf_level {
            leaves.push(stack.pop().unwrap().into_element());
        }
        leaves.reverse();

        let parent = stack.last_mut().unwrap();
        parent.set_children(leaves);
    }

    pub fn build(self) -> Option<Content> {
        #[derive(Debug)]
        enum State {
            BackIndent,
            Sibling,
            Indent,
            Empty,
        }

        let mut stack: Vec<BuilderNode> = vec![];
        for line in self.lines.into_iter() {
            if line.nodes.is_empty() {
                continue;
            }
            let node = line.process()?;

            loop {
                let state = if let Some(last) = stack.last() {
                    use std::cmp::Ordering;
                    match node.level().cmp(&last.level()) {
                        Ordering::Greater => State::Indent,
                        Ordering::Less => State::BackIndent,
                        Ordering::Equal => State::Sibling,
                    }
                } else {
                    State::Empty
                };
                match state {
                    State::BackIndent => {
                        Self::clean_stack(&mut stack);
                    }
                    State::Empty | State::Indent | State::Sibling => {
                        stack.push(node);
                        break;
                    }
                }
            }
        }
        while stack.len() > 1 {
            Self::clean_stack(&mut stack);
        }
        Some(stack.pop().unwrap().into_element())
    }
}
