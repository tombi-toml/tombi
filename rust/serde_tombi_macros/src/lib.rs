use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::quote;

#[proc_macro_attribute]
pub fn tombi(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match expand(item) {
        Ok(tokens) => tokens,
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand(item: TokenStream) -> syn::Result<TokenStream> {
    let mut item = syn::parse::<syn::Item>(item)?;
    let serde_tombi_path = serde_tombi_path()?;

    let struct_item = match &mut item {
        syn::Item::Struct(struct_item) => struct_item,
        other => {
            return Err(syn::Error::new_spanned(
                other,
                "#[serde_tombi::tombi] can only be applied to structs",
            ));
        }
    };
    let struct_ident = struct_item.ident.clone();
    let helper_mod_ident = syn::Ident::new(
        &format!("__serde_tombi_{}_helpers", struct_ident),
        Span::call_site(),
    );
    let helper_path = format!("{helper_mod_ident}::serialize_inline");

    for field in &mut struct_item.fields {
        let inline = take_tombi_inline_attr(field)?;
        if !inline {
            continue;
        }

        let serde_attrs = parse_serde_attrs(field)?;
        if serde_attrs.flatten {
            return Err(syn::Error::new_spanned(
                field,
                "#[tombi(inline)] cannot be combined with #[serde(flatten)]",
            ));
        }
        if serde_attrs.serialize_with {
            return Err(syn::Error::new_spanned(
                field,
                "#[tombi(inline)] cannot be combined with #[serde(serialize_with = ...)]",
            ));
        }
        let path = syn::LitStr::new(&helper_path, Span::call_site());

        field
            .attrs
            .push(syn::parse_quote!(#[serde(serialize_with = #path)]));
    }

    Ok(quote!(
        #[doc(hidden)]
        mod #helper_mod_ident {
            pub fn serialize_inline<T, S>(value: &T, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                T: ?Sized + serde::Serialize,
                S: serde::Serializer,
            {
                #serde_tombi_path::private::serialize_inline(value, serializer)
            }
        }

        #item
    )
    .into())
}

fn take_tombi_inline_attr(field: &mut syn::Field) -> syn::Result<bool> {
    let mut inline = false;
    let mut attrs = Vec::with_capacity(field.attrs.len());

    for attr in field.attrs.drain(..) {
        if attr.path().is_ident("tombi") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("inline") {
                    inline = true;
                    Ok(())
                } else {
                    Err(meta.error("unknown tombi attribute"))
                }
            })?;
        } else {
            attrs.push(attr);
        }
    }

    field.attrs = attrs;
    Ok(inline)
}

#[derive(Default)]
struct SerdeAttrs {
    flatten: bool,
    serialize_with: bool,
}

fn parse_serde_attrs(field: &syn::Field) -> syn::Result<SerdeAttrs> {
    let mut attrs = SerdeAttrs::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("flatten") {
                attrs.flatten = true;
            } else if meta.path.is_ident("serialize_with") {
                attrs.serialize_with = true;
                let _: syn::LitStr = meta.value()?.parse()?;
            }
            Ok(())
        })?;
    }

    Ok(attrs)
}

fn serde_tombi_path() -> syn::Result<syn::Path> {
    match crate_name("serde_tombi") {
        Ok(FoundCrate::Itself) => Ok(syn::parse_quote!(crate)),
        Ok(FoundCrate::Name(name)) => {
            let ident = syn::Ident::new(&name, Span::call_site());
            Ok(syn::parse_quote!(#ident))
        }
        Err(error) => Err(syn::Error::new(
            Span::call_site(),
            format!("failed to resolve serde_tombi crate name: {error}"),
        )),
    }
}
