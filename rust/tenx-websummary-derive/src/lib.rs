#![recursion_limit = "256"]

use std::collections::{hash_map::Entry, HashMap};

use darling::{ast, FromDeriveInput, FromField};
use form::HtmlFormReceiver;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{DeriveInput, Generics, Path};

mod form;

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(html), supports(struct_named))]
struct HtmlTemplateReceiver {
    /// The struct name.
    ident: syn::Ident,

    /// The body of the struct or enum. We don't care about enum fields
    /// because we accept only named structs. Hence the first type is null.
    data: ast::Data<(), FieldReceiver>,

    generics: Generics,

    websummary_crate: Option<Path>,
}

impl ToTokens for HtmlTemplateReceiver {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = &self.ident;
        let mut ordered_rows = Vec::new();
        let mut fields_of_row = HashMap::<String, Vec<&FieldReceiver>>::new();

        let websummary_crate = self.websummary_crate.clone().unwrap_or_else(|| {
            Path::from(proc_macro2::Ident::new(
                "tenx_websummary",
                proc_macro2::Span::call_site(),
            ))
        });

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        match self.data {
            ast::Data::Struct(ref f) => {
                for field in f.fields.iter() {
                    match fields_of_row.entry(field.row_name()) {
                        Entry::Occupied(e) => {
                            e.into_mut().push(field);
                        }
                        Entry::Vacant(v) => {
                            ordered_rows.push(v.key().clone());
                            v.insert(vec![field]);
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        let mut template_fn = quote! {
            use ::std::fmt::Write;
            use #websummary_crate::components::ReactComponent;
            let mut template = String::new();
        };
        for row in ordered_rows {
            let mut inner = quote! {};
            for field in &fields_of_row[&row] {
                let field_ident = field.ident.clone().unwrap();
                let field_ident_str = field_ident.to_string();
                inner = quote! {
                    #inner
                    let field_name = match data_key {
                        Some(ref key) => format!("{}.{}", key, #field_ident_str),
                        None => format!("{}", #field_ident_str)
                    };
                    writeln!(&mut template, r#"<div class="col">"#).unwrap();
                    writeln!(&mut template, r#"{}"#, self.#field_ident.template(Some(field_name))).unwrap();
                    writeln!(&mut template, r#"</div>"#).unwrap();
                };
            }
            template_fn = quote! {
                #template_fn
                writeln!(&mut template, r#"<div class="row">"#).unwrap();
                #inner
                writeln!(&mut template, r#"</div>"#).unwrap();
            };
        }

        tokens.append_all(quote! {
            impl #impl_generics #websummary_crate::HtmlTemplate for #ident #ty_generics #where_clause {
                fn template(&self, data_key: Option<String>) -> String {
                    #template_fn
                    template
                }
            }
        });
    }
}

#[allow(dead_code)]
#[derive(Debug, FromField)]
#[darling(attributes(html))]
struct FieldReceiver {
    /// Name of the field
    ident: Option<syn::Ident>,

    /// The type of the field
    ty: syn::Type,

    #[darling(default)]
    row: Option<String>,
}

impl FieldReceiver {
    fn row_name(&self) -> String {
        match self.row {
            Some(ref r) => r.to_string(),
            None => self.ident.as_ref().unwrap().to_string(),
        }
    }
}

const HTML_TEMPLATE_UNSUPPORTED_ERROR: &str =
    r#"HtmlTemplate can only be derived for structs with named fields"#;

#[proc_macro_derive(HtmlTemplate, attributes(html))]
pub fn html_template(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse::<DeriveInput>(item).unwrap();
    let struct_receiver = match HtmlTemplateReceiver::from_derive_input(&item) {
        Ok(r) => r,
        Err(e) => {
            return proc_macro::TokenStream::from(
                darling::Error::custom(format!("{HTML_TEMPLATE_UNSUPPORTED_ERROR}. {e}"))
                    .write_errors(),
            )
        }
    };
    quote! {
        #struct_receiver
    }
    .into()
}

const HTML_FORM_UNSUPPORTED_ERROR: &str =
    r#"HtmlForm can only be derived for structs with named fields or enum with unit variants"#;

#[proc_macro_derive(HtmlForm, attributes(html_form))]
pub fn html_form(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = syn::parse::<DeriveInput>(item).unwrap();
    let struct_receiver = match HtmlFormReceiver::from_derive_input(&item) {
        Ok(r) => r,
        Err(e) => {
            return proc_macro::TokenStream::from(
                darling::Error::custom(format!("{HTML_FORM_UNSUPPORTED_ERROR}. {e}"))
                    .write_errors(),
            )
        }
    };
    quote! {
        #struct_receiver
    }
    .into()
}

#[cfg(test)]
mod tests {
    // See https://docs.rs/trybuild/1.0.9/trybuild/ on how this test setup works
    // run `cargo test` with the environment variable `TRYBUILD=overwrite` to regenerate the
    // expected output in case you change the error message.
    // You should only use one test function.
    #[test]
    fn ui_html_template() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui_derive_html/*.rs");
    }

    #[test]
    fn ui_html_form() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui_derive_form/*.rs");
    }
}
