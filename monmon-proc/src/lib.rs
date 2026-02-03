use syn::Error;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned};

fn report_error<T: Spanned>(span: T, message: &str) -> TokenStream {
    Error::new(span.span(), message).into_compile_error().into()
}

#[proc_macro_attribute]
pub fn synchronised(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);

    let mut mon_arg = None;
    let fn_args = &input.sig.inputs;

    // find the argument that has monitor type
    // find the first argument that has the signature `&impl Monitor`
    for arg in fn_args {
        if let syn::FnArg::Typed(pat_type) = arg 
            && let syn::Type::Reference(type_ref) = &*pat_type.ty 
                && let syn::Type::ImplTrait(impl_trait) = &*type_ref.elem {
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

    // Determine which monitor to use
    let monitor_expr = if attr.is_empty() {
        // No attribute specified, try to find monitor parameter automatically
        if mon_arg.is_none() {
            return report_error(
                input.sig.paren_token.span,
                "No argument of type &impl Monitor found",
            );
        }

        // Convert the pattern to an expression for consistency
        let mon_pat = mon_arg.unwrap();
        syn::parse_quote! { #mon_pat }
    } else {
        // Parse the attribute as an expression
        let attr_str = attr.to_string();
        let attr_str = attr_str.trim();
        let attr_expr: syn::Expr = match syn::parse_str(attr_str) {
            Ok(expr) => expr,
            Err(_) => {
                return report_error(
                    input.sig.paren_token.span,
                    "Failed to parse attribute as expression",
                );
            }
        };

        // Handle different expression types
        match &attr_expr {
            syn::Expr::Path(_) => {
                // Simple path like `monitor`
                attr_expr
            }
            syn::Expr::Reference(ref_expr) => {
                // Reference like `&self.monitor` - use the inner expression
                (*ref_expr.expr).clone()
            }
            syn::Expr::Field(_) => {
                // Field access like `self.monitor` - use directly
                attr_expr
            }
            _ => {
                return report_error(
                    input.sig.paren_token.span,
                    "Attribute expression must be a path, reference, or field access",
                );
            }
        }
    };

    let block = &input.block;

    // add monitor.enter() at the begining of the block
    let enter_stmt: syn::Stmt = syn::parse_quote! {
        #monitor_expr.enter();
    };

    // add monitor.leave() at the end of the block
    let leave_stmt: syn::Stmt = syn::parse_quote! {
        #monitor_expr.leave();
    };

    let wrapped_block_stmt: syn::Stmt = syn::parse_quote! {
        #block
    };

    let block2 = syn::Block {
        brace_token: block.brace_token,
        stmts: vec![enter_stmt, wrapped_block_stmt, leave_stmt],
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
