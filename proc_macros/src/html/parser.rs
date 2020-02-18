use crate::html::Content;
use proc_macro::{token_stream, TokenTree};

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
    column: Option<usize>,
    nodes: Vec<Node>,
}

impl LineBuilder {
    // TODO: Replace Option with Result
    fn build(self) -> Option<Line> {
        Some(Line {
            level: self.level?,
            nodes: self.nodes,
        })
    }

    fn put(&mut self, token: TokenTree) {
        let span = token.span();
        if self.level.is_none() {
            self.level = Some(span.start().column);
        }

        let node = match token {
            TokenTree::Ident(ident) => Node::Ident(ident.to_string()),
            TokenTree::Punct(punct) => Node::Punct(punct.as_char()),
            TokenTree::Literal(literal) => Node::Literal(literal.to_string()),
            TokenTree::Group(group) => Node::Group(group.to_string()),
        };
        self.nodes.push(node);
        self.column = Some(span.end().column);
    }
}

#[derive(Debug)]
pub struct Line {
    level: usize,
    nodes: Vec<Node>,
}

impl Line {
    pub fn process(self) -> Option<BuilderNode> {
        #[derive(Debug)]
        enum State {
            StandBy,
            HasIdent,
            HasPropertyName(String),
            NeedPropertyValue(String),
            NeedClassName,
        };
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
                (State::StandBy, Node::Punct('.')) => {
                    tag = Some(String::from("div"));
                    state = State::NeedClassName;
                }
                (State::HasIdent, Node::Punct('.')) => {
                    state = State::NeedClassName;
                }
                (State::HasIdent, Node::Ident(ident)) => {
                    // Receive property name
                    state = State::HasPropertyName(ident);
                }
                (State::HasPropertyName(name), Node::Punct('=')) => {
                    state = State::NeedPropertyValue(name.to_string());
                }
                (State::NeedPropertyValue(name), Node::Literal(ref literal)) => {
                    properties.push((name.to_string(), literal.to_string()));
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
        Some(BuilderNode {
            level: self.level,
            tag: tag?,
            class_names,
            properties,
            contents,
            children: vec![],
        })
    }
}

#[derive(Debug)]
pub struct Parser {
    lines: Vec<Line>,
}

#[derive(Debug)]
pub struct BuilderNode {
    level: usize,
    tag: String,
    class_names: Vec<String>,
    properties: Vec<(String, String)>,
    contents: Vec<String>, // TODO: Use enum
    children: Vec<Content>,
}

impl BuilderNode {
    pub fn into_element(self) -> Content {
        let contents = if !self.contents.is_empty() {
            self.contents.into_iter().map(Content::Text).collect()
        } else {
            self.children
        };
        Content::Element {
            name: self.tag,
            class_names: self.class_names,
            properties: self.properties,
            contents,
        }
    }

    pub fn set_children(&mut self, children: Vec<Content>) {
        assert!(self.contents.is_empty(), "{:?}", self.contents);
        self.children = children;
    }
}

impl Parser {
    pub fn from_tokens(tokens: token_stream::IntoIter) -> Option<Self> {
        let mut current_line_num = None;
        let mut lines = vec![];
        let mut current_line = LineBuilder::default();

        for token in tokens {
            let span = token.span();
            let start = span.start();
            if current_line_num.map_or(false, |line_num| line_num != start.line) {
                lines.push(std::mem::take(&mut current_line).build()?);
            }
            current_line_num = Some(start.line);
            current_line.put(token);
        }
        lines.push(current_line.build()?);
        Some(Self { lines })
    }

    fn clean_stack(stack: &mut Vec<BuilderNode>) {
        let leaf_level = stack.last().unwrap().level;
        let mut leaves = vec![];
        while stack.last().unwrap().level == leaf_level {
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
            let node = line.process()?;

            loop {
                let state = if let Some(last) = stack.last() {
                    use std::cmp::Ordering;
                    match node.level.cmp(&last.level) {
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
