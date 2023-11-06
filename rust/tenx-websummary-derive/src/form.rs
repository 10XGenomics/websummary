//!
//! Helpers for procedural macro #[derive(HtmlForm)]
//!
//!

use darling::{ast, FromDeriveInput, FromField, FromMeta, FromVariant};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{Generics, Path};

#[derive(FromMeta, PartialEq, Eq, Debug, Clone, Copy)]
enum Method {
    Get,
    Post,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(html_form), supports(struct_named, enum_unit))]
pub(crate) struct HtmlFormReceiver {
    /// The struct name.
    ident: syn::Ident,
    data: ast::Data<HtmlFormVariantReceiver, HtmlFormFieldReceiver>,
    generics: Generics,
    websummary_crate: Option<Path>,
    method: Option<Method>,
    config_trait: Option<String>,
    configure: Option<bool>,
}

impl HtmlFormReceiver {
    fn ident_string(&self) -> String {
        self.ident.to_string()
    }
    fn config_trait_name(&self) -> syn::Ident {
        syn::Ident::new(
            &self
                .config_trait
                .clone()
                .unwrap_or(format!("{}Configuration", self.ident_string())),
            proc_macro2::Span::call_site(),
        )
    }
}

#[derive(Debug, FromField)]
#[darling(forward_attrs(doc))]
struct HtmlFormFieldReceiver {
    /// Name of the field
    ident: Option<syn::Ident>,
    /// The type of the field
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
}

impl HtmlFormFieldReceiver {
    fn ident_string(&self) -> String {
        self.ident.as_ref().unwrap().to_string()
    }

    fn make_title(&self, websummary_crate: &Path) -> TokenStream {
        let ident_str = self.ident_string();

        let doc_comments: Vec<_> = self
            .attrs
            .iter()
            .filter_map(|attr| {
                if let Ok(syn::Meta::NameValue(nv)) = &attr.parse_meta() {
                    if nv.path.is_ident("doc") {
                        if let syn::Lit::Str(lit_str) = &nv.lit {
                            return Some(lit_str.value().trim().to_string());
                        }
                    }
                }
                None
            })
            .filter(|x| !x.is_empty())
            .collect();

        match &doc_comments[..] {
            [] => {
                quote! {#websummary_crate::components::Title::new(#ident_str)}
            }
            [heading, rest @ ..] => {
                let help = rest.iter().join("\n");
                quote! {
                    #websummary_crate::components::Title::WithHelp(#websummary_crate::components::TitleWithHelp {
                        title: #heading.to_string(),
                        help: #help.to_string(),
                    })
                }
            }
        }
    }
    fn make_validate_fn(&self, websummary_crate: &Path) -> (syn::Ident, TokenStream) {
        let validate_fn_ident = syn::Ident::new(
            &format!("validate_{}", self.ident_string()),
            proc_macro2::Span::call_site(),
        );
        let ty = &self.ty;
        (
            validate_fn_ident.clone(),
            quote! {
                fn #validate_fn_ident(&self, value: &#ty) -> #websummary_crate::form::FieldValidationResult {
                    use #websummary_crate::form::FieldValidation;
                    <#ty as #websummary_crate::form::FieldValidation>::validate(value)
                }
            },
        )
    }
    fn make_config_fn(&self, websummary_crate: &Path) -> (syn::Ident, TokenStream) {
        let config_fn_ident = syn::Ident::new(
            &format!("configure_{}", self.ident_string()),
            proc_macro2::Span::call_site(),
        );
        let ty = &self.ty;
        (
            config_fn_ident.clone(),
            quote! {
                fn #config_fn_ident() -> <#ty as #websummary_crate::form::CreateFormInput>::Config {
                    <#ty as #websummary_crate::form::CreateFormInput>::default_config()
                }
            },
        )
    }
}

#[derive(Debug, FromVariant)]
struct HtmlFormVariantReceiver {
    /// Name of the field
    ident: syn::Ident,
}

impl ToTokens for HtmlFormReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let websummary_crate = self.websummary_crate.clone().unwrap_or_else(|| {
            Path::from(proc_macro2::Ident::new(
                "tenx_websummary",
                proc_macro2::Span::call_site(),
            ))
        });

        let struct_or_enum_ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        match self.data {
            ast::Data::Struct(ref f) => {
                let mut elements = quote! {};
                let config_trait_name = self.config_trait_name();
                let mut config_trait_impl = quote! {};
                let mut field_validations = quote! {};
                for field in &f.fields {
                    let ident = field.ident.as_ref().unwrap();
                    let ident_str = ident.to_string();
                    let title = field.make_title(&websummary_crate);
                    let (validate_fn_name, validate_fn_impl) =
                        field.make_validate_fn(&websummary_crate);
                    let (config_fn_name, config_fn_impl) = field.make_config_fn(&websummary_crate);

                    config_trait_impl = quote! {
                        #config_trait_impl
                        #validate_fn_impl
                        #config_fn_impl
                    };
                    field_validations = quote! {
                        #field_validations
                        <#struct_or_enum_ident as #config_trait_name>::#validate_fn_name(&self, &self.#ident),
                    };

                    let ty = &field.ty;
                    elements = quote! {
                        #elements
                        #websummary_crate::form::FormElement {
                            title: #title,
                            input: <#ty as #websummary_crate::form::CreateFormInput>::create_form_input(
                                <#struct_or_enum_ident as #config_trait_name>::#config_fn_name(),
                                #ident_str.to_string(),
                                value.map(|x| x.#ident.to_owned()),
                            ),
                            feedback: Default::default(),
                        },
                    }
                }
                let method = match self.method.unwrap_or(Method::Get) {
                    Method::Get => quote! { #websummary_crate::form::FormMethod::Get },
                    Method::Post => quote! { #websummary_crate::form::FormMethod::Post },
                };
                let impl_config_trait = if self.configure.unwrap_or_default() {
                    quote! {}
                } else {
                    quote! { impl #config_trait_name #ty_generics for #struct_or_enum_ident #ty_generics #where_clause {} }
                };
                tokens.append_all(quote! {
                    #[automatically_derived]
                    #[allow(clippy::all)]
                    trait #config_trait_name #ty_generics #where_clause {
                        #config_trait_impl
                    }
                    #impl_config_trait
                    #[automatically_derived]
                    impl #impl_generics #websummary_crate::form::IntoHtmlForm for #struct_or_enum_ident #ty_generics #where_clause {
                        fn _into_html_form(value: Option<&Self>) -> #websummary_crate::form::Form {
                            #websummary_crate::form::Form {
                                config: #websummary_crate::form::FormConfig {
                                    url: String::new(),
                                    method: #method,
                                },
                                elements: vec![#elements]
                            }
                        }
                        fn _field_validations(&self) -> Vec<#websummary_crate::form::FieldValidationResult> {
                            vec![
                                #field_validations
                            ]
                        }
                    }
                });
            }
            ast::Data::Enum(ref f) => {
                let mut variant_ident = quote! {};
                for variant in f {
                    let this_variant = &variant.ident;
                    variant_ident = quote! {
                        #variant_ident
                        #struct_or_enum_ident::#this_variant,
                    }
                }
                tokens.append_all(quote! {
                    impl #impl_generics #websummary_crate::form::EnumSelect for #struct_or_enum_ident #ty_generics #where_clause {
                        fn variants() -> Vec<Self> {
                            vec![#variant_ident]
                        }
                    }
                });
            }
        }
    }
}
