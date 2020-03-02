#![feature(proc_macro_span)]
extern crate proc_macro;

mod css;
mod html;
mod util;

use proc_macro::TokenStream;

#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let parser = html::Parser::from_tokens(input.into_iter()).unwrap();
    let parsed = parser.build().unwrap();
    format!("{}", parsed).parse().unwrap()
}

#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    let parser = css::Parser::from_tokens(input.into_iter()).unwrap();
    let parsed = parser.build();
    let flatten = parsed.flatten();
    format!("{}", flatten).parse().unwrap()
}
