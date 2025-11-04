use std::{collections::HashSet, marker::PhantomData};

use crate::{
    components::{ReactComponent, Title},
    react_component, HtmlTemplate,
};
use anyhow::Error;
use itertools::Itertools;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// React components in this mod
react_component!(InputFeedback, "InputFeedback");
react_component!(InputElement, "InputElement");
react_component!(SingleSelect, "SingleSelect");
react_component!(MultiSelect, "MultiSelect");
react_component!(TextArea, "TextArea");

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Input feedback

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct InputFeedback {
    pub error: Option<String>,
    pub text: Option<String>,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Input Element

#[derive(Serialize, Deserialize, Clone)]
pub struct InputElement {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: InputType,
    pub value: Option<String>,
    min: Option<String>,
    max: Option<String>,
    step: Option<String>,
    placeholder: Option<String>,
    required: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum InputType {
    Button,
    CheckBox,
    File,
    Number,
    Radio,
    Range,
    Text,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Single select

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SingleSelectType {
    Radio,
    Dropdown,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SingleSelect {
    #[serde(rename = "type")]
    pub ty: SingleSelectType,
    pub name: String,
    pub options: Vec<String>,
    pub selected: Option<String>,
    pub required: Option<bool>,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Multi select

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MultiSelectType {
    Checkbox,
    Select,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MultiSelect {
    #[serde(rename = "type")]
    pub ty: MultiSelectType,
    pub name: String,
    pub options: Vec<String>,
    pub selected: Vec<String>,
    pub required: Option<bool>,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Text area

#[derive(Serialize, Deserialize, Clone)]
pub struct TextArea {
    #[serde(skip_serializing_if = "Option::is_none")]
    rows: Option<u32>,
    pub name: String,
    placeholder: Option<String>,
    required: Option<bool>,
    pub value: Option<String>,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Spreadsheet

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpreadsheetInput {
    pub name: String,
    pub column_labels: Option<Vec<String>>,
    pub n_rows: usize,
    pub n_cols: usize,
    pub max_height: Option<String>,
    pub value: Option<String>,
}

impl ReactComponent for SpreadsheetInput {
    fn component_name() -> &'static str {
        "SpreadsheetInput"
    }
}

pub struct SpreadsheetInputConfig {
    pub column_labels: Option<Vec<String>>,
    pub n_rows: usize,
    pub n_cols: usize,
    pub max_height: Option<String>,
    pub value: Option<String>,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Wraper for all kinds of form inputs

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "content")]
pub enum FormInput {
    Input(InputElement),
    TextArea(TextArea),
    MultiSelect(MultiSelect),
    SingleSelect(SingleSelect),
    Spreadsheet(SpreadsheetInput),
}

impl FormInput {
    fn set_optional(&mut self) {
        match self {
            FormInput::Input(v) => v.required = false,
            FormInput::TextArea(v) => v.required = Some(false),
            FormInput::MultiSelect(v) => v.required = Some(false),
            FormInput::SingleSelect(v) => v.required = Some(false),
            FormInput::Spreadsheet(_) => {}
        }
    }
}

impl From<InputElement> for FormInput {
    fn from(value: InputElement) -> Self {
        FormInput::Input(value)
    }
}
impl From<TextArea> for FormInput {
    fn from(value: TextArea) -> Self {
        FormInput::TextArea(value)
    }
}
impl From<MultiSelect> for FormInput {
    fn from(value: MultiSelect) -> Self {
        FormInput::MultiSelect(value)
    }
}

impl From<SingleSelect> for FormInput {
    fn from(value: SingleSelect) -> Self {
        FormInput::SingleSelect(value)
    }
}

impl From<SpreadsheetInput> for FormInput {
    fn from(value: SpreadsheetInput) -> Self {
        FormInput::Spreadsheet(value)
    }
}

impl HtmlTemplate for FormInput {
    fn template(&self, data_key: Option<String>) -> String {
        let data_key = Some(
            [data_key, Some("content".to_string())]
                .into_iter()
                .flatten()
                .join("."),
        );
        match self {
            FormInput::Input(v) => v.template(data_key),
            FormInput::TextArea(v) => v.template(data_key),
            FormInput::MultiSelect(v) => v.template(data_key),
            FormInput::SingleSelect(v) => v.template(data_key),
            FormInput::Spreadsheet(v) => v.template(data_key),
        }
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Form element

#[derive(Serialize, Deserialize, HtmlTemplate, Clone)]
#[html(websummary_crate = "crate")]
pub struct FormElement {
    pub title: Title,
    pub input: FormInput,
    pub feedback: InputFeedback,
}

impl FormElement {
    pub fn update(&mut self, validation: FieldValidationResult) {
        match validation {
            FieldValidationResult::Valid => {}
            FieldValidationResult::Invalid { error } => {
                self.feedback.error = Some(error);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum FieldValidationResult {
    Valid,
    Invalid { error: String },
}

impl FieldValidationResult {
    pub fn new(validation: Result<(), Error>) -> Self {
        match validation {
            Ok(()) => FieldValidationResult::Valid,
            Err(e) => FieldValidationResult::Invalid {
                error: e.to_string(),
            },
        }
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Form

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FormMethod {
    Get,
    Post,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FormConfig {
    pub url: String,
    pub method: FormMethod,
}

pub enum FormValidationResult {
    Valid(Form),
    Invalid(Form),
}
impl FormValidationResult {
    pub fn inner(self) -> Form {
        match self {
            FormValidationResult::Valid(f) => f,
            FormValidationResult::Invalid(f) => f,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Form {
    pub config: FormConfig,
    pub elements: Vec<FormElement>,
}

impl HtmlTemplate for Form {
    fn template(&self, data_key: Option<String>) -> String {
        let child_data_key = data_key
            .as_ref()
            .map_or("elements".to_string(), |d| format!("{d}.elements"));
        let config_data_key = data_key
            .as_ref()
            .map_or("config".to_string(), |d| format!("{d}.config"));

        let children = self
            .elements
            .iter()
            .enumerate()
            .map(|(i, element)| element.template(Some(format!("{child_data_key}[{i}]"))))
            .join("\n");
        format!(
            r#"<div data-key="{config_data_key}" data-component="FormWrapper">
{children}
</div>"#
        )
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Traits
pub trait CreateFormInput: Sized {
    type Config;
    fn create_form_input(config: Self::Config, name: String, value: Option<Self>) -> FormInput;
    fn default_config() -> Self::Config;
    fn default_form_input(name: String, value: Option<Self>) -> FormInput {
        Self::create_form_input(Self::default_config(), name, value)
    }
}

pub enum FormInputConfigString {
    TextArea {
        rows: Option<u32>,
        placeholder: Option<String>,
    },
    Text {
        placeholder: Option<String>,
    },
}

impl CreateFormInput for String {
    type Config = FormInputConfigString;
    fn create_form_input(
        config: FormInputConfigString,
        name: String,
        value: Option<String>,
    ) -> FormInput {
        match config {
            FormInputConfigString::TextArea { rows, placeholder } => {
                FormInput::TextArea(TextArea {
                    rows,
                    name,
                    placeholder,
                    required: Some(true),
                    value,
                })
            }
            FormInputConfigString::Text { placeholder } => FormInput::Input(InputElement {
                name,
                ty: InputType::Text,
                value,
                min: None,
                max: None,
                step: None,
                placeholder,
                required: true,
            }),
        }
    }
    fn default_config() -> FormInputConfigString {
        FormInputConfigString::Text { placeholder: None }
    }
}

pub enum FormInputConfigI64 {
    Slider { min: i64, max: i64, step: i64 },
    Input { min: i64, max: i64, step: i64 },
}

impl CreateFormInput for i64 {
    type Config = FormInputConfigI64;
    fn create_form_input(
        config: FormInputConfigI64,
        name: String,
        value: Option<i64>,
    ) -> FormInput {
        let value = value.map(|x| x.to_string());
        match config {
            FormInputConfigI64::Slider { min, max, step } => FormInput::Input(InputElement {
                name,
                ty: InputType::Range,
                value,
                min: Some(min.to_string()),
                max: Some(max.to_string()),
                step: Some(step.to_string()),
                placeholder: None,
                required: true,
            }),
            FormInputConfigI64::Input { min, max, step } => FormInput::Input(InputElement {
                name,
                ty: InputType::Number,
                value,
                min: Some(min.to_string()),
                max: Some(max.to_string()),
                step: Some(step.to_string()),
                placeholder: None,
                required: true,
            }),
        }
    }

    fn default_config() -> FormInputConfigI64 {
        FormInputConfigI64::Input {
            min: i64::MIN,
            max: i64::MAX,
            step: 1,
        }
    }
}

pub trait FieldValidation {
    fn validate(&self) -> FieldValidationResult {
        FieldValidationResult::Valid
    }
}

impl FieldValidation for String {}
impl FieldValidation for i64 {}

pub trait EnumSelect: Serialize + Sized {
    fn variants() -> Vec<Self>;
    fn value(&self) -> String {
        serde_json::to_string(&self)
            .unwrap()
            .trim_matches('"')
            .to_string()
    }
    fn options() -> Vec<String> {
        Self::variants()
            .into_iter()
            .map(|v| EnumSelect::value(&v))
            .collect()
    }
}

impl<T: EnumSelect> FieldValidation for T {}

impl<T: EnumSelect> CreateFormInput for T {
    type Config = SingleSelectType;

    fn create_form_input(config: Self::Config, name: String, value: Option<Self>) -> FormInput {
        FormInput::SingleSelect(SingleSelect {
            ty: config,
            name,
            options: <T as EnumSelect>::options(),
            selected: value.map(|v| <T as EnumSelect>::value(&v)),
            required: Some(true),
        })
    }

    fn default_config() -> Self::Config {
        SingleSelectType::Radio
    }
}

impl<T: EnumSelect> FieldValidation for HashSet<T> {}

impl<T: EnumSelect> CreateFormInput for HashSet<T> {
    type Config = MultiSelectType;

    fn create_form_input(config: Self::Config, name: String, value: Option<Self>) -> FormInput {
        FormInput::MultiSelect(MultiSelect {
            ty: config,
            name,
            options: <T as EnumSelect>::options(),
            selected: value
                .iter()
                .flatten()
                .map(|v| <T as EnumSelect>::value(v))
                .collect(),
            required: Some(true),
        })
    }

    fn default_config() -> Self::Config {
        MultiSelectType::Checkbox
    }
}

pub trait IntoHtmlForm: Sized {
    fn _into_html_form(value: Option<&Self>) -> Form;
    fn _field_validations(&self) -> Vec<FieldValidationResult>;

    fn form() -> Form {
        Self::_into_html_form(None)
    }
    fn filled_form_pre_validation(&self) -> Form {
        Self::_into_html_form(Some(self))
    }
    fn validate(&self) -> FormValidationResult {
        let mut form = self.filled_form_pre_validation();
        let field_validations = self._field_validations();
        let mut invalid = false;
        for (input, validation) in form.elements.iter_mut().zip_eq(field_validations) {
            invalid |= matches!(validation, FieldValidationResult::Invalid { .. });
            input.update(validation);
        }
        if invalid {
            FormValidationResult::Invalid(form)
        } else {
            FormValidationResult::Valid(form)
        }
    }
}

#[derive(Default)]
pub struct TextAreaConfig {
    rows: Option<u32>,
    placeholder: Option<String>,
}

pub trait CsvReaderBuilder {
    fn builder() -> csv::ReaderBuilder;
}

#[derive(Debug, Clone)]
pub struct TsvNoHeader;

impl CsvReaderBuilder for TsvNoHeader {
    fn builder() -> csv::ReaderBuilder {
        let mut reader_builder = csv::ReaderBuilder::new();
        reader_builder.has_headers(false);
        reader_builder.delimiter(b'\t');
        reader_builder
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(from = "String", into = "String")]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct TableInput<T, Builder>
where
    T: Clone,
    Builder: CsvReaderBuilder + Clone,
{
    phantom: PhantomData<Builder>,
    raw_value: String,
    deserialized: Result<Vec<T>, String>,
}

impl<T, Builder> TableInput<T, Builder>
where
    T: Clone,
    Builder: CsvReaderBuilder + Clone,
{
    pub fn deserialized(self) -> Result<Vec<T>, String> {
        self.deserialized
    }
}

impl<T, Builder> CreateFormInput for TableInput<T, Builder>
where
    T: Clone,
    Builder: CsvReaderBuilder + Clone,
{
    type Config = TextAreaConfig;

    fn create_form_input(config: Self::Config, name: String, value: Option<Self>) -> FormInput {
        FormInput::TextArea(TextArea {
            rows: config.rows,
            name,
            placeholder: config.placeholder,
            required: Some(true),
            value: value.map(|x| x.raw_value),
        })
    }

    fn default_config() -> Self::Config {
        TextAreaConfig::default()
    }
}

impl<T, Builder> From<String> for TableInput<T, Builder>
where
    T: Clone + DeserializeOwned,
    Builder: CsvReaderBuilder + Clone,
{
    fn from(src: String) -> Self {
        let deserialized: Result<Vec<T>, _> = Builder::builder()
            .from_reader(src.as_bytes())
            .deserialize()
            .try_collect()
            .map_err(|e| e.to_string());
        TableInput {
            raw_value: src,
            deserialized,
            phantom: PhantomData,
        }
    }
}

impl<T, Builder> From<TableInput<T, Builder>> for String
where
    T: Clone + DeserializeOwned,
    Builder: CsvReaderBuilder + Clone,
{
    fn from(src: TableInput<T, Builder>) -> String {
        src.raw_value
    }
}

impl<T, Builder> FieldValidation for TableInput<T, Builder>
where
    T: Clone + DeserializeOwned,
    Builder: CsvReaderBuilder + Clone,
{
    fn validate(&self) -> FieldValidationResult {
        match &self.deserialized {
            Ok(_) => FieldValidationResult::Valid,
            Err(e) => FieldValidationResult::Invalid { error: e.clone() },
        }
    }
}

impl<T: CreateFormInput> CreateFormInput for Option<T> {
    type Config = T::Config;

    fn create_form_input(config: Self::Config, name: String, value: Option<Self>) -> FormInput {
        let mut input = T::create_form_input(config, name, value.flatten());
        input.set_optional();
        input
    }

    fn default_config() -> Self::Config {
        T::default_config()
    }
}

impl<T: FieldValidation> FieldValidation for Option<T> {
    fn validate(&self) -> FieldValidationResult {
        match self {
            Some(v) => v.validate(),
            None => FieldValidationResult::Valid,
        }
    }
}

/// Hack because the csv crate does not expose this explicitly
pub fn tabular_file_header<T>() -> Result<Vec<String>, Error>
where
    T: Serialize + Default,
{
    let mut buffer = Vec::new();
    let mut wtr = csv::WriterBuilder::default()
        .has_headers(true)
        .from_writer(&mut buffer);
    // The header row is written automatically.
    wtr.serialize(T::default())?;
    wtr.flush()?;
    drop(wtr);
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(buffer.as_slice());
    let headers = rdr.headers()?;
    Ok(headers
        .iter()
        .map(std::string::ToString::to_string)
        .collect())
}

pub trait ConfigureSpreadsheet: Serialize + Default {
    fn max_height() -> Option<String> {
        None
    }
    fn num_rows() -> usize {
        10
    }
    fn initial_value() -> Option<Vec<Self>>
    where
        Self: Sized,
    {
        None
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(from = "String", into = "String")]
#[serde(bound = "T: Serialize + DeserializeOwned")]
pub struct Spreadsheet<T: Clone + ConfigureSpreadsheet> {
    pub input: TableInput<T, TsvNoHeader>,
}

impl<T> From<String> for Spreadsheet<T>
where
    T: Clone + ConfigureSpreadsheet + DeserializeOwned,
{
    fn from(src: String) -> Self {
        Spreadsheet {
            input: TableInput::from(src),
        }
    }
}

impl<T> From<Spreadsheet<T>> for String
where
    T: Clone + ConfigureSpreadsheet + DeserializeOwned,
{
    fn from(src: Spreadsheet<T>) -> String {
        src.input.into()
    }
}

impl<T> Spreadsheet<T>
where
    T: Clone + ConfigureSpreadsheet + DeserializeOwned,
{
    pub fn deserialized(self) -> Result<Vec<T>, String> {
        self.input.deserialized()
    }
}

impl<T> CreateFormInput for Spreadsheet<T>
where
    T: Clone + ConfigureSpreadsheet + DeserializeOwned + Serialize,
{
    type Config = SpreadsheetInputConfig;

    fn create_form_input(config: Self::Config, name: String, value: Option<Self>) -> FormInput {
        let value = value.map(|x| x.input.raw_value).or(config.value);
        FormInput::Spreadsheet(SpreadsheetInput {
            name,
            column_labels: config.column_labels,
            n_rows: config.n_rows,
            n_cols: config.n_cols,
            max_height: config.max_height,
            value,
        })
    }

    fn default_config() -> Self::Config {
        let column_labels = tabular_file_header::<T>().unwrap();
        let n_cols = column_labels.len();
        SpreadsheetInputConfig {
            column_labels: Some(column_labels),
            n_rows: T::num_rows(),
            n_cols,
            max_height: T::max_height(),
            value: T::initial_value().map(|x| {
                let mut wtr = csv::WriterBuilder::new()
                    .delimiter(b'\t')
                    .has_headers(false)
                    .from_writer(vec![]);
                for row in x {
                    wtr.serialize(row).unwrap();
                }
                String::from_utf8(wtr.into_inner().unwrap()).unwrap()
            }),
        }
    }
}

impl<T> FieldValidation for Spreadsheet<T>
where
    T: Clone + ConfigureSpreadsheet + DeserializeOwned + Serialize,
{
    fn validate(&self) -> FieldValidationResult {
        self.input.validate()
    }
}
