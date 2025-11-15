//! # Half Macros
//!
//! Procedural macros for the Half web framework.
//! Provides routing and middleware attribute macros.

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, LitStr, Token, parse::Parse, parse::ParseStream, parse_macro_input};

mod route;

/// HTTP method enum for route macros
#[derive(Debug, Clone, Copy)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl HttpMethod {
    fn as_str(self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }
}

/// Parse route macro arguments: method, path
struct RouteArgs {
    method: HttpMethod,
    path: String,
}

impl Parse for RouteArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method_ident: syn::Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let path: LitStr = input.parse()?;

        let method = match method_ident.to_string().to_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            _ => {
                return Err(syn::Error::new_spanned(
                    method_ident,
                    "Invalid HTTP method. Use GET, POST, PUT, DELETE, PATCH, HEAD, or OPTIONS",
                ));
            }
        };

        Ok(RouteArgs {
            method,
            path: path.value(),
        })
    }
}

/// Main route macro: `#[route(GET, "/path")]`
///
/// # Example
/// ```ignore
/// #[route(GET, "/users")]
/// async fn get_users(req: Request) -> Response {
///     // handler implementation
/// }
/// ```
#[proc_macro_attribute]
pub fn route(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as RouteArgs);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_asyncness = &input.sig.asyncness;
    let fn_attrs = &input.attrs;

    let method = args.method.as_str();
    let path = args.path;

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_asyncness fn #fn_name(#fn_inputs) #fn_output {
            #fn_block
        }

        // Generate route metadata
        impl #fn_name {
            pub const METHOD: &'static str = #method;
            pub const PATH: &'static str = #path;
        }
    };

    TokenStream::from(expanded)
}

/// GET route macro: `#[get("/path")]`
#[proc_macro_attribute]
pub fn get(args: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(args as LitStr);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_asyncness = &input.sig.asyncness;
    let fn_attrs = &input.attrs;

    let path_str = path.value();

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_asyncness fn #fn_name(#fn_inputs) #fn_output {
            #fn_block
        }

        // Generate route metadata
        impl #fn_name {
            pub const METHOD: &'static str = "GET";
            pub const PATH: &'static str = #path_str;
        }
    };

    TokenStream::from(expanded)
}

/// POST route macro: `#[post("/path")]`
#[proc_macro_attribute]
pub fn post(args: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(args as LitStr);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_asyncness = &input.sig.asyncness;
    let fn_attrs = &input.attrs;

    let path_str = path.value();

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_asyncness fn #fn_name(#fn_inputs) #fn_output {
            #fn_block
        }

        impl #fn_name {
            pub const METHOD: &'static str = "POST";
            pub const PATH: &'static str = #path_str;
        }
    };

    TokenStream::from(expanded)
}

/// PUT route macro: `#[put("/path")]`
#[proc_macro_attribute]
pub fn put(args: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(args as LitStr);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_asyncness = &input.sig.asyncness;
    let fn_attrs = &input.attrs;

    let path_str = path.value();

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_asyncness fn #fn_name(#fn_inputs) #fn_output {
            #fn_block
        }

        impl #fn_name {
            pub const METHOD: &'static str = "PUT";
            pub const PATH: &'static str = #path_str;
        }
    };

    TokenStream::from(expanded)
}

/// DELETE route macro: `#[delete("/path")]`
#[proc_macro_attribute]
pub fn delete(args: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(args as LitStr);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_asyncness = &input.sig.asyncness;
    let fn_attrs = &input.attrs;

    let path_str = path.value();

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_asyncness fn #fn_name(#fn_inputs) #fn_output {
            #fn_block
        }

        impl #fn_name {
            pub const METHOD: &'static str = "DELETE";
            pub const PATH: &'static str = #path_str;
        }
    };

    TokenStream::from(expanded)
}

/// PATCH route macro: `#[patch("/path")]`
#[proc_macro_attribute]
pub fn patch(args: TokenStream, item: TokenStream) -> TokenStream {
    let path = parse_macro_input!(args as LitStr);
    let input = parse_macro_input!(item as ItemFn);

    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_asyncness = &input.sig.asyncness;
    let fn_attrs = &input.attrs;

    let path_str = path.value();

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_asyncness fn #fn_name(#fn_inputs) #fn_output {
            #fn_block
        }

        impl #fn_name {
            pub const METHOD: &'static str = "PATCH";
            pub const PATH: &'static str = #path_str;
        }
    };

    TokenStream::from(expanded)
}
