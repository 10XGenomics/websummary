// #![deny(missing_docs)]

//! Generate 10x HTML websummary from provided data.

/// Code to generate html from the json data
pub mod generate_html;

use std::collections::HashMap;

#[cfg(feature = "generate_html")]
pub use generate_html::generate_html_summary;

pub use generate_html::{
    generate_html_summary_with_build_files, TemplateInfo, WebSummaryBuildFiles,
};

use components::WsNavBar;
use serde::{Deserialize, Serialize};

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate tenx_websummary_derive;
use serde_json::Value;
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
    #[serde(default, rename = "_resources")]
    resources: SharedResources,
}

pub const RESOURCES_PREFIX: &str = "_resources";
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SharedResources(pub HashMap<String, Value>);

impl SharedResources {
    pub fn new() -> Self {
        SharedResources::default()
    }
    pub fn insert(&mut self, value: Value) -> String {
        // Check if the value is already in the map
        let key = match self
            .0
            .iter()
            .find_map(|(key, val)| (val == &value).then(|| key.clone()))
        {
            Some(key) => key,
            None => {
                let key = format!("{:03}", self.0.len());
                self.0.insert(key.clone(), value);
                key
            }
        };
        format!("{}_{}", RESOURCES_PREFIX, key)
    }
}

pub trait AddToSharedResource {
    fn add_to_shared_resource(&mut self, shared_resource: &mut SharedResources);
    fn with_shared_resource(mut self, shared_resource: &mut SharedResources) -> Self
    where
        Self: Sized,
    {
        self.add_to_shared_resource(shared_resource);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AlertLevel {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub title: String,
    pub formatted_value: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Default, Deserialize)]
pub struct Alerts {
    #[serde(rename = "alarms")]
    pub alerts: Vec<Alert>,
}

impl<P> SinglePageHtml<P> {
    pub fn from_content(content: P) -> Self {
        SinglePageHtml {
            nav_bar: None,
            alerts: Alerts::default(),
            content,
            config: SinglePageConfig::default(),
            resources: SharedResources::new(),
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
            resources: SharedResources::new(),
        }
    }
    pub fn full_width(mut self) -> Self {
        self.config = self.config.full_width();
        self
    }
    pub fn resources(mut self, resources: SharedResources) -> Self {
        self.resources = resources;
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
        build_files: WebSummaryBuildFiles<'_>,
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
        build_files: WebSummaryBuildFiles<'_>,
    ) -> Result<(), anyhow::Error> {
        let writer = std::io::BufWriter::new(std::fs::File::create(file)?);
        self.generate_html_with_build_files(writer, build_files)
    }
}
