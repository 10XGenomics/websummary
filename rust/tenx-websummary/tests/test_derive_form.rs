#![cfg(feature = "form")]
use std::collections::HashSet;

use serde::Serialize;
use tenx_websummary::form::{
    EnumSelect, FieldValidationResult, FormMethod, IntoHtmlForm, SingleSelectType,
};
use tenx_websummary_derive::HtmlForm;

#[test]
fn test_enum_derive() {
    #[derive(Serialize, HtmlForm, Debug, PartialEq)]
    enum Scaling {
        Log,
        Linear,
    }

    assert_eq!(Scaling::variants(), vec![Scaling::Log, Scaling::Linear]);
    assert_eq!(
        Scaling::options(),
        vec!["Log".to_string(), "Linear".to_string()]
    );
}

#[test]
fn test_enum_derive_rename() {
    #[derive(Serialize, HtmlForm, Debug, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum Scaling {
        Log,
        Linear,
    }

    assert_eq!(Scaling::variants(), vec![Scaling::Log, Scaling::Linear]);
    assert_eq!(
        Scaling::options(),
        vec!["log".to_string(), "linear".to_string()]
    );
}

#[test]
fn test_struct_derive_get() {
    #[derive(Serialize, HtmlForm, Debug, PartialEq)]
    struct MyForm {
        analysis_id: i64,
    }

    let form = MyForm::form();
    assert_eq!(form.config.method, FormMethod::Get);
    assert_eq!(form.elements.len(), 1);
    insta::assert_ron_snapshot!(form);
}

#[test]
fn test_struct_derive() {
    #[derive(Serialize, HtmlForm, Debug, PartialEq, Clone, Hash, Eq)]
    enum Scaling {
        Log,
        Linear,
    }

    #[derive(Serialize, HtmlForm, Debug, PartialEq, Eq)]
    #[html_form(method = "post", configure)]
    struct MyForm {
        analysis_id: i64,
        /// Metric
        ///
        /// Enter a metric
        metric: String,
        scaling: Scaling,
        scaling_set: HashSet<Scaling>,
    }

    impl MyFormConfiguration for MyForm {
        fn validate_analysis_id(&self, analysis_id: &i64) -> FieldValidationResult {
            if *analysis_id < 10000 {
                FieldValidationResult::Invalid {
                    error: "Too small an analysis id".into(),
                }
            } else {
                FieldValidationResult::Valid
            }
        }
        fn configure_scaling() -> SingleSelectType {
            SingleSelectType::Radio
        }
    }

    let form = MyForm {
        analysis_id: 1000,
        metric: "filtered_bcs".into(),
        scaling: Scaling::Log,
        scaling_set: [Scaling::Linear].into_iter().collect(),
    }
    .validate()
    .inner();
    assert_eq!(form.config.method, FormMethod::Post);
    assert_eq!(form.elements.len(), 4);
    insta::assert_ron_snapshot!(form);
}
