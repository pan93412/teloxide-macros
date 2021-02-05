use syn::{Type, ItemFn, FnArg, ReturnType};
use syn::parse::{Parse, ParseStream};
use syn::parse_macro_input;
use proc_macro::TokenStream;
use crate::compile_error;

pub enum TeloxideAttribute {
    Subtransition
}

impl Parse for TeloxideAttribute {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        match input.parse::<syn::Ident>()?.to_string().as_str() {
            "subtransition" => {
                Ok(TeloxideAttribute::Subtransition)
            }
            _ => unreachable!()
        }
    }
}

pub fn expand(attr: TeloxideAttribute, item: TokenStream) -> TokenStream {
    match attr {
        TeloxideAttribute::Subtransition => {
            let item = parse_macro_input!(item as ItemFn);
            expand_subtransition_fn(item)
        }
    }
}

fn expand_subtransition_fn(item: ItemFn) -> TokenStream {
    let params = item.sig.inputs.iter().collect::<Vec<&FnArg>>();

    let dialogue = match params.get(0) {
        Some(data_param_type) => match *data_param_type {
            FnArg::Typed(typed) => typed.ty.clone(),
            _ => return compile_error("`self` is not allowed."),
        },
        None => {
            return compile_error("expected 2 arguments, found 0");
        }
    };
    let aux_param_type = match params.get(2) {
        Some(data_param_type) => match *data_param_type {
            FnArg::Typed(typed) => typed.ty.clone(),
            _ => unreachable!(),
        },
        None => {
            let unit_type = proc_macro::TokenStream::from(quote::quote! {()});
            Box::new(parse_macro_input!(unit_type as Type))
        }
    };
    let err = match &item.sig.output {
        ReturnType::Type(_, ty) => quote::quote! { <#ty as SubtransitionOutputType>::Error },
        ReturnType::Default => quote::quote! {
            std::convert::Infallible
        }
    };

    let fn_name = &item.sig.ident;
    let call_fn = match params.get(2) {
        Some(_) => {
            quote::quote! {  #fn_name(self, cx, aux) }
        }
        None => quote::quote! { #fn_name(self, cx) },
    };
    let call_fn = match &item.sig.output {
        ReturnType::Type(_, _) => call_fn,
        ReturnType::Default => quote::quote! {
            #call_fn.map(|_| Ok(()))
        }
    };

    (quote::quote! {
        const _: () = {
            use teloxide::dispatching::dialogue::{Subtransition, SubtransitionOutputType, SubtransitionState, TransitionIn};

            struct Hack;

            impl Subtransition<Hack> for #dialogue {
                type Aux = #aux_param_type;
                type Error = #err;

                fn react(self, cx: TransitionIn, aux: #aux_param_type)
                    -> futures::future::BoxFuture<'static, TransitionOut<<Self as Subtransition<Hack>>::Error>> {
                            #item
                            Box::pin(#call_fn)
                        }
            }
        };
    }).into()
}