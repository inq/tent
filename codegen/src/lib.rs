mod css;
mod html;
mod util;

use proc_macro::TokenStream;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn html(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let parser = html::Parser::from_str(input.value().as_str()).unwrap();
    let parsed = parser.build().unwrap();
    format!("{}", parsed).parse().unwrap()
}

#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let parser = css::Parser::from_str(input.value().as_str()).unwrap();
    let parsed = parser.build();
    let flatten = parsed.flatten();
    format!("{}", flatten).parse().unwrap()
}
