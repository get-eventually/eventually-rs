//! `eventually-macros` contains useful macros that provides
//! different implementations of traits and functionalities from [eventually].

#![deny(unsafe_code, unused_qualifications, trivial_casts, missing_docs)]
#![deny(clippy::all, clippy::pedantic, clippy::cargo)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Fields, ItemStruct, Meta, NestedMeta, Path};

/// Implements a newtype to use the [`eventually::aggregate::Root`] instance with
/// user-defined [`eventually::aggregate::Aggregate`] types.
///
/// # Context
///
/// The eventually API uses [`aggregate::Root`][eventually::aggregate::Root]
/// to manage the versioning and list of events to commit for an `Aggregate` instance.
/// Domain commands are to be implemented on the `aggregate::Root<T>` instance, as it gives
/// access to use `Root<T>.record_that` or `Root<T>.record_new` to record Domain Events.
///
/// However, it's not possible to use `impl aggregate::Root<MyAggregateType>` (`MyAggregateType`
/// being an example of user-defined `Aggregate` type) outside the `eventually` crate (E0116).
/// Therefore, a newtype that uses `aggregate::Root<T>` is required.
///
/// This attribute macro makes the implementation of a newtype easy, as it Implements
/// conversion traits from and to `aggregate::Root<T>` and implements automatic deref
/// through [`std::ops::Deref`] and [`std::ops::DerefMut`].
///
/// # Panics
///
/// This method will panic if the Aggregate Root type is not provided as a macro parameter.
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
