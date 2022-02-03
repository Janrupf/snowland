use proc_macro2::TokenStream;

use crate::attr::CallHandlerArgumentAttribute;
use syn::{FnArg, ImplItem, ImplItemMethod, Index, ItemImpl, Visibility};

pub fn process_call_handler(input: TokenStream) -> TokenStream {
    let im = match syn::parse2::<ItemImpl>(input) {
        Ok(v) => v,
        Err(err) => {
            proc_macro_error::emit_error!(err.span(), err);
            return quote::quote! {};
        }
    };

    let cleaned = clean_input(&im);
    let wrapper = generate_wrapper(&im);

    quote::quote! {
        #cleaned

        #wrapper
    }
}

fn clean_input(im: &ItemImpl) -> ItemImpl {
    let mut out = im.clone();

    for item in &mut out.items {
        if let ImplItem::Method(method) = item {
            for arg in &mut method.sig.inputs {
                if let FnArg::Typed(t) = arg {
                    CallHandlerArgumentAttribute::clean(&mut t.attrs);
                }
            }
        }
    }

    out
}

fn generate_wrapper(im: &ItemImpl) -> TokenStream {
    let generics = &im.generics;
    let im_name = &im.self_ty;

    let arms = im.items.iter().filter_map(|item| {
        if let ImplItem::Method(method) = item {
            generate_arm(method)
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

fn generate_arm(method: &ImplItemMethod) -> Option<TokenStream> {
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

    let mut arg_pos = 0;
    let mut has_responder = false;

    let (args, tuple_types): (Vec<_>, Vec<_>) = method
        .sig
        .inputs
        .iter()
        .filter_map(|i| match i {
            FnArg::Receiver(_) => None,
            FnArg::Typed(t) => Some(t),
        })
        .map(|t| {
            let attrs = CallHandlerArgumentAttribute::from_attrs(&t.attrs);

            if attrs.contains(&CallHandlerArgumentAttribute::Engine) {
                (quote::quote! { engine }, None)
            } else if attrs.contains(&CallHandlerArgumentAttribute::Responder) {
                has_responder = true;
                (quote::quote! { responder }, None)
            } else {
                let index = Index::from(arg_pos);
                arg_pos += 1;

                (quote::quote! { args.#index }, Some(&t.ty))
            }
        })
        .unzip();

    let tuple_types = tuple_types.into_iter().flatten().collect::<Vec<_>>();

    let call_prefix = if requires_self {
        quote::quote! { self. }
    } else {
        quote::quote! { Self:: }
    };

    let call = quote::quote! {
        #call_prefix #name (#(#args,)*)
    };

    let arguments_type = if tuple_types.is_empty() {
        quote::quote! { type Args = () }
    } else {
        quote::quote! {
            #[derive(::serde::Deserialize)]
            struct Args(#(#tuple_types,)*)
        }
    };

    let call_handler = if has_responder {
        quote::quote! {
            let _: () = #call
        }
    } else {
        quote::quote! {
            let res: Result<_, _> = #call;
            responder.result(res)
        }
    };

    let responder_constructor = if !has_responder {
        quote::quote! { crate::com::Responder::<crate::com::DirectInnerResponder>::new(&call, reply) }
    } else {
        quote::quote! {
            {
                let context = nativeshell::shell::Context::current().unwrap();
                let sender = context.run_loop.borrow().new_sender();

                crate::com::Responder::<crate::com::ThreadSafeInnerResponder>::new(
                    &call,
                    reply,
                    sender,
                )
            }
        }
    };

    Some(quote::quote! {
        #name_str => {
            #arguments_type;

            let responder = #responder_constructor;
            let args: Args = match crate::util::reserialize(call.args) {
                Ok(v) => v,
                Err(err) => {
                    responder.failed(
                        "INVALID_ARGS",
                        Some(format!(
                            "Failed to convert arguments for method {}: {}",
                            call.method,
                            err
                        )),
                        Some(::nativeshell::codec::Value::String(call.method))
                    );
                    return;
                }
            };

            #call_handler;
        }
    })
}
