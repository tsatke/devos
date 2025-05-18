use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_quote, Attribute, ItemFn, LitStr};

// Define the base URL for the POSIX specification.
const BASE_URL: &str = "https://pubs.opengroup.org/onlinepubs/9799919799";

#[proc_macro_attribute]
pub fn posix_spec(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute argument, which is the specific link part.
    // This expects the attribute to be used like #[posix_spec = "read.html"]
    let link_part_lit = match syn::parse::<LitStr>(attr.clone()) {
        Ok(lit) => lit,
        Err(_) => {
            // If parsing LitStr directly fails, it might be due to `attr` being `ident = "string"`
            // This is a common way attributes are structured, e.g. `#[posix_spec(path = "open.html")]`
            // However, the request is `#[posix_spec = "<some link part>"]`
            // For `name = value` syntax, `attr` contains only `value`.
            // `parse_macro_input!` is a helper that provides better error messages.
            let parsed_attr = parse_macro_input!(attr as LitStr);
            parsed_attr
        }
    };
    let link_part = link_part_lit.value();

    // Parse the item the attribute is attached to (expected to be a function).
    let mut item_fn = parse_macro_input!(item as ItemFn);

    let full_url = format!("{BASE_URL}/{link_part}");

    let doc_line1 = format!(" See [`{link_part}`] in the POSIX spec for details.");
    let doc_line3 = format!(" [`{link_part}`]: {full_url}");

    let attr1: Attribute = parse_quote!(#[doc = #doc_line1]);
    let attr2: Attribute = parse_quote!(#[doc = ""]);
    let attr3: Attribute = parse_quote!(#[doc = #doc_line3]);

    let mut new_attrs = vec![attr1, attr2, attr3];
    new_attrs.append(&mut item_fn.attrs);
    item_fn.attrs = new_attrs;

    // Return the modified function as a TokenStream.
    TokenStream::from(quote!(#item_fn))
}
