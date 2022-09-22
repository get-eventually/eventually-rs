use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, Fields, ItemStruct, Meta, NestedMeta, Path};

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
