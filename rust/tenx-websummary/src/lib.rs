// #![deny(missing_docs)]

//! Generate 10x HTML websummary from provided data.

/// Code to generate html from the json data
pub mod generate_html;

#[cfg(feature = "generate_html")]
pub use generate_html::generate_html_summary;

pub use generate_html::{
    generate_html_summary_with_build_files, TemplateInfo, WebSummaryBuildFiles,
};

use components::WsNavBar;
use serde::Serialize;

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate tenx_websummary_derive;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use tenx_websummary_derive::*;

pub mod macros;

/// Websummary components
pub mod components;

#[cfg(feature = "image_base64_encode")]
pub mod image_base64_encode;

#[cfg(feature = "image_proc")]
pub mod image_proc;

#[cfg(feature = "image_proc")]
pub use image;

#[cfg(feature = "csv_table")]
pub mod csv_table;

#[cfg(feature = "actix")]
pub mod actix;

pub mod scrape_json;

#[cfg(feature = "form")]
pub mod form;

pub trait HtmlTemplate {
    fn template(&self, data_key: Option<String>) -> String;
}

#[derive(Debug, Clone)]
struct SinglePageConfig {
    div_class: String,
}

impl Default for SinglePageConfig {
    fn default() -> Self {
        SinglePageConfig {
            div_class: "container".into(),
        }
    }
}

impl SinglePageConfig {
    pub fn full_width(mut self) -> Self {
        self.div_class = "container-fluid".into();
        self
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct SinglePageHtml<P> {
    #[serde(rename = "sample")]
    nav_bar: Option<WsNavBar>,
    #[serde(flatten)]
    content: P,
    #[serde(rename = "alarms")]
    alerts: Alerts,
    #[serde(skip)]
    config: SinglePageConfig,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AlertLevel {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub title: String,
    pub formatted_value: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Alerts {
    #[serde(rename = "alarms")]
    alerts: Vec<Alert>,
}

impl<P> SinglePageHtml<P> {
    pub fn from_content(content: P) -> Self {
        SinglePageHtml {
            nav_bar: None,
            alerts: Alerts::default(),
            content,
            config: SinglePageConfig::default(),
        }
    }
    pub fn nav_bar(mut self, nav_bar: WsNavBar) -> Self {
        self.nav_bar = Some(nav_bar);
        self
    }
    pub fn alerts(mut self, alerts: Vec<Alert>) -> Self {
        self.alerts = Alerts { alerts };
        self
    }
    pub fn new(nav_bar: WsNavBar, content: P, alerts: Option<Vec<Alert>>) -> Self {
        SinglePageHtml {
            nav_bar: Some(nav_bar),
            content,
            alerts: Alerts {
                alerts: alerts.unwrap_or_default(),
            },
            config: SinglePageConfig::default(),
        }
    }
    pub fn full_width(mut self) -> Self {
        self.config = self.config.full_width();
        self
    }
}
impl<P: HtmlTemplate> HtmlTemplate for SinglePageHtml<P> {
    fn template(&self, data_key: Option<String>) -> String {
        let div_nav_bar = self.nav_bar.as_ref().map_or("", |_| {
            r#"<div class="navbar-wrapper"></div>
<div class="namescription-wrapper"></div>"#
        });
        format!(
            r#"{div_nav_bar}
<div class="alert-wrapper"></div>
<div class="{}">{}</div>
"#,
            self.config.div_class,
            self.content.template(data_key)
        )
    }
}

impl<P: Serialize + HtmlTemplate> SinglePageHtml<P> {
    #[cfg(feature = "generate_html")]
    pub fn generate_html<W: std::io::Write>(self, writer: W) -> Result<(), anyhow::Error> {
        let json_data = serde_json::to_string(&self)?;

        generate_html_summary(
            &json_data,
            self.template(None),
            TemplateInfo::<String>::Default,
            writer,
        )
    }

    #[cfg(feature = "generate_html")]
    pub fn generate_html_file(
        self,
        file: impl AsRef<std::path::Path>,
    ) -> Result<(), anyhow::Error> {
        let writer = std::io::BufWriter::new(std::fs::File::create(file)?);
        self.generate_html(writer)
    }

    pub fn generate_html_with_build_files<W: std::io::Write>(
        self,
        writer: W,
        build_files: WebSummaryBuildFiles,
    ) -> Result<(), anyhow::Error> {
        let json_data = serde_json::to_string(&self)?;

        generate_html_summary_with_build_files(
            &json_data,
            self.template(None),
            TemplateInfo::<String>::Default,
            writer,
            build_files,
        )
    }

    pub fn generate_html_file_with_build_files(
        self,
        file: impl AsRef<std::path::Path>,
        build_files: WebSummaryBuildFiles,
    ) -> Result<(), anyhow::Error> {
        let writer = std::io::BufWriter::new(std::fs::File::create(file)?);
        self.generate_html_with_build_files(writer, build_files)
    }
}
