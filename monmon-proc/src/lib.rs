use quote::{quote, ToTokens};
use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn synchronised(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);

    let mut mon_arg = None;

    let fn_args = &input.sig.inputs;

    // find the argument that has monitor type 
    // find the first argument that has the signature `&impl Monitor`
    for arg in fn_args {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Type::Reference(type_ref) = &*pat_type.ty {
                if let syn::Type::ImplTrait(impl_trait) = &*type_ref.elem
                {
                    for bound in &impl_trait.bounds {
                        if let syn::TypeParamBound::Trait(trait_bound) = bound {
                            let path = &trait_bound.path;
                            if path.is_ident("Monitor") {
                                mon_arg = Some(pat_type.pat.clone());
                            }
                        }
                    }
                }   
            }
        }
    }

    if mon_arg.is_none() {
        panic!("No argument of type &impl Monitor found");
    }

    let mon_arg = mon_arg.unwrap();
    let block = &input.block;

    // add monitor.enter() at the begining of the block
    let enter_stmt: syn::Stmt = syn::parse_quote! {
        #mon_arg.enter();
    };

    // add monitor.leave() at the end of the block
    let leave_stmt: syn::Stmt = syn::parse_quote! {
        #mon_arg.leave();
    };

    let wrapped_block_stmt: syn::Stmt = syn::parse_quote! {
        #block
    };

    let block2 = syn::Block {
        brace_token: block.brace_token,
        stmts: vec![
            enter_stmt,
            wrapped_block_stmt,
            leave_stmt,
        ]
    };

    let output = syn::ItemFn {
        attrs: input.attrs.clone(),
        vis: input.vis.clone(),
        sig: input.sig.clone(),
        block: Box::new(block2),
    };

    // eprintln!("{}", output.to_token_stream().to_string());

    quote!(#output).into()
}
