mod call_handler;

use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream};

struct Empty;

impl Parse for Empty {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if !input.is_empty() {
            Err(syn::Error::new(input.span(), "Expected no tokens"))
        } else {
            Ok(Empty)
        }
    }
}

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn method_channel_call_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Err(err) = syn::parse::<Empty>(attr) {
        proc_macro_error::emit_error!(
            err.span(),
            "method_channel_call_handler macro takes not attributes";
            help = err.span() => "Remove these tokens"
        );
    }

    let out = call_handler::process_call_handler(item.into()).into();

    println!("{}", out);

    out
}
