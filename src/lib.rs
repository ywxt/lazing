#![feature(proc_macro_diagnostic)]
#![feature(allow_internal_unstable)]

//! A macro like lazy_static can initialize static variables.
//!
//! # Usage
//!
//! ```
//! # use std::ops::Deref;
//! #[lazy]
//! static NAME: String = "Hello".to_string();
//!
//! fn main() {
//!    println!("{}",NAME.deref());
//! }
//!  
//! ```

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, Ident, Token, Type, Visibility};

struct LazyStatic {
    visibility: Visibility,
    name: Ident,
    ty: Type,
    init: Expr,
}

impl Parse for LazyStatic {
    fn parse(input: ParseStream) -> Result<Self> {
        let visibility: Visibility = input.parse()?;
        input.parse::<Token![static]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty: Type = input.parse()?;
        input.parse::<Token![=]>()?;
        let init: Expr = input.parse()?;
        input.parse::<Token![;]>()?;
        Ok(LazyStatic {
            visibility,
            name,
            ty,
            init,
        })
    }
}

/// Parses the following syntax
/// ```
/// # const IGNORE_TOKENS: &str = stringify! {
/// #[lazy]
/// $Visibility static $NAME: $Type = $EXPRESS;
/// # };
/// ```
/// # Example
/// ```
/// # const IGNORE_TOKENS: &str = stringify! {
/// #[lazy]
/// pub static foo: String = "Hello".to_string();
/// # };
/// ```
#[proc_macro_attribute]
pub fn lazy(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        proc_macro2::TokenStream::from(attr)
            .span()
            .unwrap()
            .error("no parameter should be at here.")
            .emit();
        return TokenStream::new();
    }
    let LazyStatic {
        visibility,
        name,
        ty,
        init,
    } = parse_macro_input!(item as LazyStatic);

    // Assert that the static type implements Sync. If not, user sees an error
    // message like the following. We span this assertion with the field type's
    // line/column so that the error message appears in the correct place.
    //
    //     error[E0277]: the trait bound `*const (): std::marker::Sync` is not satisfied
    //       --> src/main.rs:10:21
    //        |
    //     10 |     static ref PTR: *const () = &();
    //        |                     ^^^^^^^^^ `*const ()` cannot be shared between threads safely
    let assert_sync = quote_spanned! {ty.span()=>
        struct _AssertSync where #ty: std::marker::Sync;
    };

    // Check for Sized. Not vital to check here, but the error message is less
    // confusing this way than if they get a Sized error in one of our
    // implementation details where it assumes Sized.
    //
    //     error[E0277]: the trait bound `str: std::marker::Sized` is not satisfied
    //       --> src/main.rs:10:19
    //        |
    //     10 |     static ref A: str = "";
    //        |                   ^^^ `str` does not have a constant size known at compile-time
    let assert_sized = quote_spanned! {ty.span()=>
        struct _AssertSized where #ty: std::marker::Sized;
    };

    let init_ptr = quote_spanned! {init.span()=>
        Box::into_raw(Box::new(#init))
    };

    let expanded = quote! {
        #[allow(non_camel_case_types)]
        #visibility struct #name;

        impl std::ops::Deref for #name {
            type Target = #ty;

            fn deref(&self) -> &#ty {
                #assert_sync
                #assert_sized

                static ONCE: std::sync::Once = std::sync::Once::new();
                static mut VALUE: *mut #ty = 0 as *mut #ty;

                unsafe {
                    ONCE.call_once(|| VALUE = #init_ptr);
                    &*VALUE
                }
            }
        }
    };

    TokenStream::from(expanded)
}
