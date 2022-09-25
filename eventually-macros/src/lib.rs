//! Module containing useful macros for the [eventually] crate.

#![deny(unsafe_code, unused_qualifications, trivial_casts)]
#![warn(missing_docs)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Fields, ItemEnum, ItemStruct, Meta, NestedMeta, Path};

#[proc_macro_derive(Message)]
pub fn derive_message(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemEnum);
    let item_name = item.ident;
    let event_prefix = item_name
        .to_string()
        .strip_suffix("Event")
        .unwrap()
        .to_owned();

    let match_cases = item.variants.iter().fold(quote! {}, |acc, variant| {
        let event_type = &variant.ident;
        let event_name = format!("{}{}", event_prefix, event_type);

        quote! {
            #acc
            #item_name::#event_type { .. } => #event_name,
        }
    });

    let result = quote! {
        impl eventually::message::Message for #item_name {
            fn name(&self) -> &'static str {
                match self {
                    #match_cases
                }
            }
        }
    };

    result.into()
}

/// Implements a newtype to use the [eventually::aggregate::Root] instance with
/// user-defined [eventually::aggregate::Aggregate] types.
///
/// # Context
///
/// The eventually API uses `aggregate::Root<T>` to manage the versioning and
/// list of events to commit for an `Aggregate` instance. Domain commands
/// are to be implemented on the `aggregate::Root<T>` instance, as it gives
/// access to use `Root<T>.record_that` or `Root<T>.record_new` to record Domain Events.
///
/// However, it's not possible to use `impl aggregate::Root<MyAggregateType>` (`MyAggregateType`
/// being an example of user-defined `Aggregate` type) outside the `eventually` crate (E0116).
/// Therefore, a newtype that uses `aggregate::Root<T>` is required.
///
/// This attribute macro makes the implementation of a newtype easy, as it Implements
/// conversion traits from and to `aggregate::Root<T>` and implements automatic deref
/// through [std::ops::Deref] and [std::ops::DerefMut].
#[proc_macro_attribute]
pub fn aggregate_root(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let mut item = parse_macro_input!(item as ItemStruct);
    let item_ident = item.ident.clone();

    let aggregate_type = args
        .first()
        .and_then(|meta| match meta {
            NestedMeta::Meta(Meta::Path(Path { segments, .. })) => Some(segments),
            _ => None,
        })
        .and_then(|segments| segments.first())
        .map(|segment| segment.ident.clone())
        .expect("the aggregate root type must be provided as macro parameter");

    item.fields = Fields::Unnamed(
        syn::parse2(quote! { (eventually::aggregate::Root<#aggregate_type>) }).unwrap(),
    );

    let result = quote! {
        #item

        impl std::ops::Deref for #item_ident {
            type Target = eventually::aggregate::Root<#aggregate_type>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for #item_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl From<eventually::aggregate::Root<#aggregate_type>> for #item_ident {
            fn from(root: eventually::aggregate::Root<#aggregate_type>) -> Self {
                Self(root)
            }
        }

        impl From<#item_ident> for eventually::aggregate::Root<#aggregate_type> {
            fn from(value: #item_ident) -> Self {
                value.0
            }
        }
    };

    result.into()
}
