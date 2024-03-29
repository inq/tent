use std::str::FromStr;

use crate::css::{Content, Item};
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
    pub fn process(self) -> Option<BuilderNode> {
        #[derive(Debug)]
        enum State {
            StandBy,
            HasPrefix(char),
            HasIdent(String),
            HasAccumulatedIdent(String),
            HasAccumulatedPunct(String),
            NeedDeclarationValue(String),
            Done,
        }
        let mut state = State::StandBy;
        let mut res = None;

        for node in self.nodes.into_iter() {
            match (&state, node) {
                (State::StandBy, Node::Punct('@')) => state = State::HasPrefix('@'),
                (State::StandBy, Node::Ident(ident)) => state = State::HasIdent(ident),
                (State::StandBy, Node::Punct('.')) => {
                    state = State::HasAccumulatedPunct(".".to_string())
                }
                (State::StandBy, Node::Punct('#')) => {
                    state = State::HasAccumulatedPunct("#".to_string())
                }
                (State::HasPrefix(prefix), Node::Ident(ref ident)) => {
                    state = State::HasIdent(format!(
                        "{}{}",
                        prefix,
                        crate::util::camelcase_to_dashed(ident)
                    ))
                }
                (State::HasIdent(ident), Node::Punct('.')) => {
                    state = State::HasAccumulatedPunct(format!("{} .", ident))
                }
                (State::HasIdent(ident), Node::Punct(':')) => {
                    // Declaration
                    state = State::NeedDeclarationValue(ident.to_string())
                }
                (State::NeedDeclarationValue(ref ident), Node::Literal(ref literal)) => {
                    res = Some(Item::Declaration(
                        crate::util::camelcase_to_dashed(ident),
                        literal.to_string(),
                    ));
                    state = State::Done;
                }
                (State::HasIdent(prev), Node::Ident(ref ident)) => {
                    state = State::HasAccumulatedIdent(format!("{} {}", prev, ident))
                }
                (State::HasAccumulatedIdent(prev), Node::Ident(ref ident)) => {
                    state = State::HasAccumulatedIdent(format!("{} {}", prev, ident))
                }
                (State::HasAccumulatedPunct(prev), Node::Ident(ref ident)) => {
                    state = State::HasAccumulatedIdent(format!("{}{}", prev, ident))
                }
                (State::HasAccumulatedIdent(prev), Node::Punct(ref ident)) => {
                    state = State::HasAccumulatedPunct(format!("{} {}", prev, ident))
                }
                (State::HasAccumulatedPunct(prev), Node::Punct(ref ident)) => {
                    state = State::HasAccumulatedPunct(format!("{}{}", prev, ident))
                }
                (state, node) => {
                    panic!("{:?}, {:?}", state, node);
                }
            }
        }

        match state {
            State::HasIdent(ident) => {
                res = Some(Item::Node {
                    name: ident,
                    children: vec![],
                });
            }
            State::HasAccumulatedIdent(ident) => {
                res = Some(Item::Node {
                    name: ident,
                    children: vec![],
                });
            }
            State::Done => (),
            state => {
                panic!("{:?}", state);
            }
        }

        if let Some(res) = res {
            Some(BuilderNode {
                level: self.level,
                inner: res,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct Parser {
    lines: Vec<Line>,
}

#[derive(Debug)]
pub struct BuilderNode {
    level: usize,
    inner: Item,
}

impl BuilderNode {
    pub fn set_children(&mut self, children_new: Vec<Item>) {
        match &mut self.inner {
            Item::Node {
                ref mut children, ..
            } => {
                *children = children_new;
            }
            etc => panic!("{:?}", etc),
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

    fn clean_stack(stack: &mut Vec<BuilderNode>) -> Vec<Item> {
        let leaf_level = stack.last().unwrap().level;
        let mut leaves = vec![];
        while !stack.is_empty() && stack.last().unwrap().level == leaf_level {
            leaves.push(stack.pop().unwrap().inner);
        }
        leaves.reverse();
        leaves
    }

    pub fn build(self) -> Content {
        #[derive(Debug)]
        enum State {
            BackIndent,
            Sibling,
            Indent,
            Empty,
        }

        let mut stack: Vec<BuilderNode> = vec![];
        let mut res = vec![];

        for line in self.lines.into_iter() {
            if line.nodes.is_empty() {
                continue;
            }
            let node = line.process().unwrap();

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
                        let mut siblings = Self::clean_stack(&mut stack);
                        if stack.is_empty() {
                            res.append(&mut siblings);
                        } else {
                            let parent = stack.last_mut().unwrap();
                            parent.set_children(siblings);
                        }
                    }
                    State::Empty | State::Indent | State::Sibling => {
                        stack.push(node);
                        break;
                    }
                }
            }
        }
        while !stack.is_empty() {
            let mut siblings = Self::clean_stack(&mut stack);
            if stack.is_empty() {
                res.append(&mut siblings);
            } else {
                let parent = stack.last_mut().unwrap();
                parent.set_children(siblings);
            }
        }
        Content { items: res }
    }
}
