use proc_macro2::TokenStream;

use syn::{FnArg, ImplItem, ImplItemMethod, Index, ItemImpl, Visibility};

pub fn process_call_handler(input: TokenStream) -> TokenStream {
    let wrapper = generate_wrapper(input.clone());

    quote::quote! {
        #input

        #wrapper
    }
}

fn generate_wrapper(input: TokenStream) -> TokenStream {
    let im = match syn::parse2::<ItemImpl>(input) {
        Ok(v) => v,
        Err(err) => {
            proc_macro_error::emit_error!(err.span(), err);
            return quote::quote! {};
        }
    };

    let generics = &im.generics;
    let im_name = &im.self_ty;

    let arms = im.items.iter().filter_map(|item| {
        if let ImplItem::Method(method) = item {
            generate_arm(&im, method)
        } else {
            None
        }
    });

    quote::quote! {
        impl #generics ::nativeshell::shell::MethodCallHandler for #im_name #generics {
            fn on_method_call(
                &mut self,
                call: ::nativeshell::codec::MethodCall<::nativeshell::codec::Value>,
                reply: ::nativeshell::codec::MethodCallReply<::nativeshell::codec::Value>,
                engine: ::nativeshell::shell::EngineHandle,
            ) {
                match call.method.as_str() {
                    #(#arms,)*
                    _ => {
                        reply.send_error(
                            "NOT_IMPLEMENTED",
                            Some(&format!("Method {} is not implemented", call.method)),
                            ::nativeshell::codec::Value::String(call.method),
                        );
                    }
                }
            }
        }
    }
}

fn generate_arm(tr: &ItemImpl, method: &ImplItemMethod) -> Option<TokenStream> {
    if !matches!(method.vis, Visibility::Public(_)) {
        return None;
    }

    let name = &method.sig.ident;
    let name_str = name.to_string();

    let requires_self = method
        .sig
        .inputs
        .iter()
        .any(|i| matches!(i, FnArg::Receiver(_)));

    let input_types = method
        .sig
        .inputs
        .iter()
        .filter_map(|i| match i {
            FnArg::Receiver(_) => None,
            FnArg::Typed(t) => Some(&t.ty),
        })
        .collect::<Vec<_>>();

    let call_prefix = if requires_self {
        quote::quote! { self. }
    } else {
        quote::quote! { Self:: }
    };

    let call_args = (0..input_types.len())
        .map(Index::from)
        .map(|i| quote::quote! { args.#i });

    let call = quote::quote! {
        #call_prefix #name (#(#call_args,)*)
    };

    let arguments_type = if input_types.is_empty() {
        quote::quote! { type Args = () }
    } else {
        quote::quote! {
            #[derive(::serde::Deserialize)]
            struct Args(#(#input_types,)*)
        }
    };

    Some(quote::quote! {
        #name_str => {
            #arguments_type;

            let args: Args = match crate::util::reserialize(call.args) {
                Ok(v) => v,
                Err(err) => {
                    reply.send_error(
                        "INVALID_ARGS",
                        Some(&format!(
                            "Failed to convert arguments for method {}: {}",
                            call.method,
                            err
                        )),
                        ::nativeshell::codec::Value::String(call.method),
                    );
                    return;
                }
            };

            let res: Result<_, Box<dyn std::error::Error>> = #call.map_err(std::convert::Into::into);

            match res {
                Ok(v) => {
                    match crate::util::reserialize(v) {
                        Ok(v) => reply.send_ok(v),
                        Err(err) => {
                            reply.send_error(
                                "INVALID_RETURN_VALUE",
                                Some(&format!(
                                    "Failed to convert the return value of method {}: {}",
                                    call.method,
                                    err
                                )),
                                ::nativeshell::codec::Value::String(call.method)
                            )
                        }
                    }
                },
                Err(err) => {
                    reply.send_error(
                        "EXECUTION_FAILED",
                        Some(&err.to_string()),
                        ::nativeshell::codec::Value::Null
                    )
                }
            };
        }
    })
}
