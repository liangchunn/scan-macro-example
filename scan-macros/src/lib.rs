use proc_macro::TokenStream;
use scan_core::{generate, MacroInput};
use syn::parse_macro_input;

#[proc_macro]
pub fn scan(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MacroInput);
    generate(input).into()
}
