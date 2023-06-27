use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::ItemFn;

pub fn expand(test_fn: ItemFn) -> TokenStream {
    let fn_name_ident = &test_fn.sig.ident;
    let fn_name = fn_name_ident.to_string();

    let description_name = format_ident!("__KERNEL_TEST_{}", fn_name);

    let name = quote! { #fn_name };

    let test_location = quote! {
        kernel_test_framework::SourceLocation {
            module: module_path!(),
            file: file!(),
            line: line!(),
            column: column!(),
        }
    };

    quote! {
        #test_fn

        #[linkme::distributed_slice(kernel_test_framework::KERNEL_TESTS)]
        static #description_name: kernel_test_framework::KernelTestDescription = kernel_test_framework::KernelTestDescription {
            name: #name,
            test_fn: #fn_name_ident,
            test_location: #test_location,
        };
    }
}
