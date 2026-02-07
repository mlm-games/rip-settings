use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

/// Parsed `#[category(...)]` attributes.
struct CategoryAttrs {
    order: i32,
    title: Option<String>,
}

pub fn expand_category(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let attrs = parse_category_attrs(&input)?;

    let order = attrs.order;
    let title = attrs.title.unwrap_or_else(|| name.to_string());

    Ok(quote! {
        impl #name {
            /// Category display order.
            pub const CATEGORY_ORDER: i32 = #order;
            /// Category display title.
            pub const CATEGORY_TITLE: &'static str = #title;
        }
    })
}

fn parse_category_attrs(input: &DeriveInput) -> Result<CategoryAttrs> {
    let mut order = 0i32;
    let mut title: Option<String> = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("category") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("order") {
                let value = meta.value()?;
                let lit: syn::LitInt = value.parse()?;
                order = lit.base10_parse()?;
            } else if meta.path.is_ident("title") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                title = Some(lit.value());
            }
            Ok(())
        })?;
    }

    Ok(CategoryAttrs { order, title })
}
