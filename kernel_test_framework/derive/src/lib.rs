use proc_macro::TokenStream;
use syn::parse_macro_input;

mod declaration;

#[proc_macro_attribute]
pub fn kernel_test(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    let expanded = declaration::expand(parse_macro_input!(item));

    TokenStream::from(expanded)
}
