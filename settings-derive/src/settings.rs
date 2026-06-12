use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Result};

struct SettingsAttrs {
    version: u32,
    app_id: String,
}

#[derive(Default)]
struct FieldAttrs {
    title: String,
    description: String,
    category: String,
    category_order: i32,
    kind: String,
    key: Option<String>,
    depends_on: Option<String>,
    persisted_only: bool,
    no_reset: bool,
    confirm_reset: Option<String>,
    platforms: Vec<String>,
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    options: Vec<String>,
    action: Option<String>,
    confirm_title: Option<String>,
    confirm_message: Option<String>,
    confirm_text: Option<String>,
    cancel_text: Option<String>,
    is_dangerous: bool,
}

#[derive(Default)]
struct ValidateAttrs {
    range_min: Option<f64>,
    range_max: Option<f64>,
    length_min: Option<usize>,
    length_max: Option<usize>,
    pattern: Option<String>,
    required: bool,
    error_message: Option<String>,
}

pub fn expand_settings(input: DeriveInput) -> Result<TokenStream> {
    let name = &input.ident;
    let settings_attrs = parse_settings_attrs(&input)?;

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named) => &named.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "Settings can only be derived on structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "Settings can only be derived on structs",
            ));
        }
    };

    let version = settings_attrs.version;
    let app_id = &settings_attrs.app_id;

    let mut field_meta_entries = Vec::new();
    let mut get_arms = Vec::<proc_macro2::TokenStream>::new();
    let mut set_arms = Vec::<proc_macro2::TokenStream>::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        let field_attrs = parse_field_attrs(field)?;
        let validate_attrs = parse_validate_attrs(field)?;

        let key_name = field_attrs
            .key
            .clone()
            .unwrap_or_else(|| field_name_str.to_case(Case::Snake));

        get_arms.push(quote! {
            #field_name_str => {
                serde_json::to_value(&self.#field_name)
                    .map_err(|e| multiplatform_settings_core::error::SettingsError::Serialization(e))
            }
        });

        set_arms.push(quote! {
            #field_name_str => {
                self.#field_name = serde_json::from_value(value)
                    .map_err(|e| multiplatform_settings_core::error::SettingsError::Serialization(e))?;
                Ok(())
            }
        });

        let kind_tokens = build_field_kind(&field_attrs);
        let validation_tokens = build_validation(&validate_attrs);
        let confirmation_tokens = build_confirmation(&field_attrs);

        let depends_on_tokens = match &field_attrs.depends_on {
            Some(dep) => quote! { Some(#dep) },
            None => quote! { None },
        };

        let confirm_reset_tokens = match &field_attrs.confirm_reset {
            Some(msg) => quote! { Some(#msg) },
            None => quote! { None },
        };

        let platform_strs = &field_attrs.platforms;
        let platforms_tokens = if platform_strs.is_empty() {
            quote! { Vec::new() }
        } else {
            quote! { vec![#(#platform_strs),*] }
        };

        let title = &field_attrs.title;
        let description = &field_attrs.description;
        let category = &field_attrs.category;
        let category_order = field_attrs.category_order;
        let persisted_only = field_attrs.persisted_only;
        let no_reset = field_attrs.no_reset;
        let key_str = &key_name;

        field_meta_entries.push(quote! {
            multiplatform_settings_core::field::FieldMeta {
                name: #field_name_str,
                key: #key_str,
                title: #title,
                description: #description,
                category: #category,
                category_order: #category_order,
                kind: #kind_tokens,
                depends_on: #depends_on_tokens,
                is_persisted_only: #persisted_only,
                no_reset: #no_reset,
                confirm_reset: #confirm_reset_tokens,
                validation: #validation_tokens,
                confirmation: #confirmation_tokens,
                platforms: #platforms_tokens,
            }
        });
    }

    Ok(quote! {
        impl multiplatform_settings_core::schema::SettingsSchema for #name {
            fn schema_version(&self) -> u32 {
                #version
            }

            fn app_id(&self) -> &'static str {
                #app_id
            }

            fn fields(&self) -> Vec<multiplatform_settings_core::field::FieldMeta> {
                vec![
                    #(#field_meta_entries),*
                ]
            }

            fn get_field_value(&self, name: &str) -> Result<serde_json::Value, multiplatform_settings_core::error::SettingsError> {
                match name {
                    #(#get_arms)*
                    _ => Err(multiplatform_settings_core::error::SettingsError::UnknownField(name.to_string())),
                }
            }

            fn set_field_value(&mut self, name: &str, value: serde_json::Value) -> Result<(), multiplatform_settings_core::error::SettingsError> {
                match name {
                    #(#set_arms)*
                    _ => Err(multiplatform_settings_core::error::SettingsError::UnknownField(name.to_string())),
                }
            }
        }
    })
}

fn parse_settings_attrs(input: &DeriveInput) -> Result<SettingsAttrs> {
    let mut version = 1u32;
    let mut app_id = String::new();

    for attr in &input.attrs {
        if !attr.path().is_ident("settings") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("version") {
                let value = meta.value()?;
                let lit: syn::LitInt = value.parse()?;
                version = lit.base10_parse()?;
            } else if meta.path.is_ident("app_id") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                app_id = lit.value();
            }
            Ok(())
        })?;
    }

    Ok(SettingsAttrs { version, app_id })
}

fn parse_field_attrs(field: &syn::Field) -> Result<FieldAttrs> {
    let mut attrs = FieldAttrs::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("setting") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("persisted_only") {
                attrs.persisted_only = true;
                return Ok(());
            }
            if meta.path.is_ident("no_reset") {
                attrs.no_reset = true;
                return Ok(());
            }
            if meta.path.is_ident("is_dangerous") {
                attrs.is_dangerous = true;
                return Ok(());
            }

            if meta.path.is_ident("title") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.title = lit.value();
            } else if meta.path.is_ident("description") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.description = lit.value();
            } else if meta.path.is_ident("category") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.category = lit.value();
            } else if meta.path.is_ident("category_order") {
                let value = meta.value()?;
                let lit: syn::LitInt = value.parse()?;
                attrs.category_order = lit.base10_parse()?;
            } else if meta.path.is_ident("kind") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.kind = lit.value();
            } else if meta.path.is_ident("key") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.key = Some(lit.value());
            } else if meta.path.is_ident("depends_on") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.depends_on = Some(lit.value());
            } else if meta.path.is_ident("min") {
                let value = meta.value()?;
                let lit: syn::LitFloat = value.parse()?;
                attrs.min = Some(lit.base10_parse()?);
            } else if meta.path.is_ident("max") {
                let value = meta.value()?;
                let lit: syn::LitFloat = value.parse()?;
                attrs.max = Some(lit.base10_parse()?);
            } else if meta.path.is_ident("step") {
                let value = meta.value()?;
                let lit: syn::LitFloat = value.parse()?;
                attrs.step = Some(lit.base10_parse()?);
            } else if meta.path.is_ident("options") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.options = lit
                    .value()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
            } else if meta.path.is_ident("action") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.action = Some(lit.value());
            } else if meta.path.is_ident("confirm_title") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.confirm_title = Some(lit.value());
            } else if meta.path.is_ident("confirm_message") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.confirm_message = Some(lit.value());
            } else if meta.path.is_ident("confirm_text") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.confirm_text = Some(lit.value());
            } else if meta.path.is_ident("cancel_text") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.cancel_text = Some(lit.value());
            } else if meta.path.is_ident("confirm_reset") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.confirm_reset = Some(lit.value());
            } else if meta.path.is_ident("platforms") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.platforms = lit
                    .value()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
            }
            Ok(())
        })?;
    }

    Ok(attrs)
}

fn parse_validate_attrs(field: &syn::Field) -> Result<ValidateAttrs> {
    let mut attrs = ValidateAttrs::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("validate") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("required") {
                attrs.required = true;
            } else if meta.path.is_ident("range") {
                meta.parse_nested_meta(|inner| {
                    if inner.path.is_ident("min") {
                        let value = inner.value()?;
                        let lit: syn::LitFloat = value.parse()?;
                        attrs.range_min = Some(lit.base10_parse()?);
                    } else if inner.path.is_ident("max") {
                        let value = inner.value()?;
                        let lit: syn::LitFloat = value.parse()?;
                        attrs.range_max = Some(lit.base10_parse()?);
                    }
                    Ok(())
                })?;
            } else if meta.path.is_ident("length") {
                meta.parse_nested_meta(|inner| {
                    if inner.path.is_ident("min") {
                        let value = inner.value()?;
                        let lit: syn::LitInt = value.parse()?;
                        attrs.length_min = Some(lit.base10_parse()?);
                    } else if inner.path.is_ident("max") {
                        let value = inner.value()?;
                        let lit: syn::LitInt = value.parse()?;
                        attrs.length_max = Some(lit.base10_parse()?);
                    }
                    Ok(())
                })?;
            } else if meta.path.is_ident("pattern") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.pattern = Some(lit.value());
            } else if meta.path.is_ident("error") {
                let value = meta.value()?;
                let lit: syn::LitStr = value.parse()?;
                attrs.error_message = Some(lit.value());
            }
            Ok(())
        })?;
    }

    Ok(attrs)
}

fn build_field_kind(attrs: &FieldAttrs) -> TokenStream {
    match attrs.kind.as_str() {
        "toggle" | "" => quote! { multiplatform_settings_core::field::FieldKind::Toggle },
        "slider" => {
            let min = attrs.min.unwrap_or(0.0);
            let max = attrs.max.unwrap_or(100.0);
            let step = attrs.step.unwrap_or(1.0);
            quote! {
                multiplatform_settings_core::field::FieldKind::Slider {
                    min: #min,
                    max: #max,
                    step: #step,
                }
            }
        }
        "dropdown" => {
            let opts = &attrs.options;
            quote! {
                multiplatform_settings_core::field::FieldKind::Dropdown {
                    options: vec![#(#opts.to_string()),*],
                }
            }
        }
        "text" | "text_input" => {
            quote! { multiplatform_settings_core::field::FieldKind::TextInput }
        }
        "button" => {
            let action = attrs.action.as_deref().unwrap_or("");
            quote! {
                multiplatform_settings_core::field::FieldKind::Button {
                    action: #action.to_string(),
                }
            }
        }
        other => {
            let name = other.to_string();
            quote! {
                multiplatform_settings_core::field::FieldKind::Custom {
                    type_name: #name.to_string(),
                }
            }
        }
    }
}

fn build_validation(validate: &ValidateAttrs) -> TokenStream {
    let has_validation = validate.range_min.is_some()
        || validate.range_max.is_some()
        || validate.length_min.is_some()
        || validate.length_max.is_some()
        || validate.pattern.is_some()
        || validate.required;

    if !has_validation {
        return quote! { None };
    }

    let range_tokens = if validate.range_min.is_some() || validate.range_max.is_some() {
        let min = validate.range_min.unwrap_or(f64::MIN);
        let max = validate.range_max.unwrap_or(f64::MAX);
        quote! { Some((#min, #max)) }
    } else {
        quote! { None }
    };

    let length_tokens = if validate.length_min.is_some() || validate.length_max.is_some() {
        let min = validate.length_min.unwrap_or(0);
        let max = validate.length_max.unwrap_or(usize::MAX);
        quote! { Some((#min, #max)) }
    } else {
        quote! { None }
    };

    let pattern_tokens = match &validate.pattern {
        Some(p) => quote! { Some(#p.to_string()) },
        None => quote! { None },
    };

    let required = validate.required;
    let error_msg_tokens = match &validate.error_message {
        Some(msg) => quote! { Some(#msg.to_string()) },
        None => quote! { None },
    };

    quote! {
        Some(multiplatform_settings_core::field::ValidationRules {
            range: #range_tokens,
            length: #length_tokens,
            pattern: #pattern_tokens,
            required: #required,
            error_message: #error_msg_tokens,
        })
    }
}

fn build_confirmation(attrs: &FieldAttrs) -> TokenStream {
    let has_confirmation =
        attrs.confirm_title.is_some() || attrs.confirm_message.is_some() || attrs.is_dangerous;

    if !has_confirmation {
        return quote! { None };
    }

    let title = attrs.confirm_title.as_deref().unwrap_or("Confirm Change");
    let message = attrs
        .confirm_message
        .as_deref()
        .unwrap_or("Are you sure you want to change this setting?");
    let confirm_text = attrs.confirm_text.as_deref().unwrap_or("Confirm");
    let cancel_text = attrs.cancel_text.as_deref().unwrap_or("Cancel");
    let is_dangerous = attrs.is_dangerous;

    quote! {
        Some(multiplatform_settings_core::field::ConfirmationConfig {
            title: #title.to_string(),
            message: #message.to_string(),
            confirm_text: #confirm_text.to_string(),
            cancel_text: #cancel_text.to_string(),
            is_dangerous: #is_dangerous,
        })
    }
}
