#![feature(proc_macro_span)]
extern crate proc_macro;

mod content;
mod parser;

use parser::Parser;
use proc_macro::TokenStream;

#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let parser = Parser::from_tokens(input.into_iter()).unwrap();
    let parsed = parser.build().unwrap();
    format!("{}", parsed).parse().unwrap()
}
