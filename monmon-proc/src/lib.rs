use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn synchronised(_attr: TokenStream, item: TokenStream) -> TokenStream {
    unimplemented!()
}
