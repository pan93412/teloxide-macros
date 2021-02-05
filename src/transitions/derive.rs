use syn::{ItemEnum, Fields, Type};
use crate::compile_error;
use proc_macro::TokenStream;

pub fn expand(item: ItemEnum) -> TokenStream {
    let enum_name = &item.ident;
    let field_type_of_first_variant =
        match &item.variants.iter().next().unwrap().fields {
            Fields::Unnamed(fields) => {
                &fields
                    .unnamed
                    .iter()
                    .next()
                    // .unwrap() because empty enumerations are not yet allowed
                    // in stable Rust.
                    .unwrap()
                    .ty

            }
            _ => return compile_error("Only one unnamed field per variant is allowed"),
        };

    let variant = item.variants.iter().map(|x|&x.ident);

    (quote::quote! {
        const _: () = {
            use teloxide::dispatching::dialogue::{Transition, Subtransition, Dialogue, TransitionIn, TransitionOut, SubtransitionState, SubtransitionHack};
            use futures::future::BoxFuture;

            impl<E> Transition for Dialogue<#enum_name, E> {
                type Aux = <#field_type_of_first_variant as Subtransition<<#field_type_of_first_variant as SubtransitionHack>::HackType>>::Aux;
                type Error = <#field_type_of_first_variant as Subtransition<<#field_type_of_first_variant as SubtransitionHack>::HackType>>::Error;

                fn react(self, cx: TransitionIn, aux: Self::Aux)
                    -> BoxFuture<'static, TransitionOut<<Self as Transition>::Error>>
                {
                    match &self.data {
                        #(
                        Self::#variant(state) => {
                            let dialogue = self.map(|d| match d { Self::#variant(s) => s, _ => unreachable!() });
                            Subtransition::react(dialogue, cx, aux)
                        },
                        )*
                    }
                }
            }
        };
    }).into()
}