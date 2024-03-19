use std::io::prelude::*;
use std::path::Path;
use std::{borrow::Cow, fs::read_to_string};

use anyhow::{format_err, Error};
use regex::Regex;

pub struct WebSummaryBuildFiles<'a> {
    pub script_js: Cow<'a, str>,
    pub styles_css: Cow<'a, str>,
    pub template_html: Cow<'a, str>,
}

impl WebSummaryBuildFiles<'_> {
    pub fn new(script_js: String, styles_css: String, template_html: String) -> Self {
        WebSummaryBuildFiles {
            script_js: Cow::Owned(script_js),
            styles_css: Cow::Owned(styles_css),
            template_html: Cow::Owned(template_html),
        }
    }
    #[cfg(feature = "generate_html")]
    fn _generated() -> Self {
        use tenx_websummary_build::{SCRIPT, STYLES, TEMPLATE};
        WebSummaryBuildFiles {
            script_js: SCRIPT.into(),
            styles_css: STYLES.into(),
            template_html: TEMPLATE.into(),
        }
    }
}

/// Possible ways to load template data
pub enum TemplateInfo<P: AsRef<Path> = String> {
    /// Use the default (bundled) template
    Default,
    /// Load the template.html from the provided directory, if it exists, otherwise use the default
    /// (bundled) template
    Dynamic(P),
    /// Use the template source provided herein
    Static(String),
}

/// Writes compiled all-in-one HTML of the websummary, returning an error if any.
///
/// # Arguments
///
/// * `json_data` - A string slice holding the data for the summary in JSON form
/// * `summary_contents` - A String holding the summary html for page, absent templating
/// * `template_dir` - An optional Path to additional template components
/// * `writer` - The Writer to which the all-in-one HTML will be written
#[cfg(feature = "generate_html")]
pub fn generate_html_summary<P, W>(
    json_data: &str,
    summary_contents: String,
    template_info: TemplateInfo<P>,
    writer: W,
) -> Result<(), Error>
where
    P: AsRef<Path>,
    W: Write,
{
    generate_html_summary_with_build_files(
        json_data,
        summary_contents,
        template_info,
        writer,
        WebSummaryBuildFiles::_generated(),
    )
}

/// Writes compiled all-in-one HTML of the websummary, returning an error if any.
///
/// # Arguments
///
/// * `json_data` - A string slice holding the data for the summary in JSON form
/// * `summary_contents` - A String holding the summary html for page, absent templating
/// * `template_dir` - An optional Path to additional template components
/// * `writer` - The Writer to which the all-in-one HTML will be written
/// * `script_js, styles_css, template` - Web summary build artifacts
pub fn generate_html_summary_with_build_files<P, W>(
    json_data: &str,
    mut summary_contents: String,
    template_info: TemplateInfo<P>,
    mut writer: W,
    WebSummaryBuildFiles {
        script_js,
        styles_css,
        template_html,
    }: WebSummaryBuildFiles<'_>,
) -> Result<(), Error>
where
    P: AsRef<Path>,
    W: Write,
{
    let (template_dir, mut template_src) = match template_info {
        TemplateInfo::Default => (None, String::from(template_html)),
        TemplateInfo::Dynamic(path) => {
            let template = path.as_ref().join("template.html");
            (
                Some(path),
                if template.exists() {
                    read_to_string(template)?
                } else {
                    String::from(template_html)
                },
            )
        }
        TemplateInfo::Static(template_src) => (None, template_src),
    };
    let re = Regex::new(r"\[\[ include (?P<filename>[a-zA-Z./_\d-]+) \]\]").unwrap();
    let mut count = 0;
    loop {
        if count > 100 {
            return Err(format_err!("Maximum recursion depth exceeded!"));
        }
        count += 1;
        if let Some(m) = re.captures(&summary_contents) {
            if let Some(template_dir) = template_dir.as_ref() {
                let path = template_dir
                    .as_ref()
                    .join(m.name("filename").unwrap().as_str());
                let src = read_to_string(path)?;
                summary_contents = summary_contents.replace(m.get(0).unwrap().as_str(), &src);
            } else {
                return Err(format_err!(
                    "found replacement {} but template_dir is None",
                    m.get(0).unwrap().as_str()
                ));
            }
        } else {
            break;
        }
    }

    for (from, to) in &[
        ("[[ tenx-websummary-script.min.js ]]", script_js),
        ("[[ tenx-websummary-styles.min.css ]]", styles_css),
        ("[[ data.js ]]", json_data.into()),
        ("[[ summary.html ]]", summary_contents.into()),
    ] {
        template_src = template_src.replace(from, to);
    }

    writer.write_all(template_src.as_bytes())?;

    Ok(())
}

#[cfg(all(test, feature = "generate_html"))]
mod tests {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn generate_html_example() {
        let json_data = read_to_string("../../example/data.json").unwrap();
        let contents = read_to_string("../../example/summary.html").unwrap();
        let template_info = TemplateInfo::Dynamic("../../example");
        // let mut out = File::create("test.html").unwrap();
        let mut out: Vec<u8> = vec![];
        assert!(generate_html_summary(&json_data, contents, template_info, &mut out).is_ok());
        assert!(!out.is_empty());
    }

    #[test]
    fn generate_html_cellranger() {
        let json_data = read_to_string("../../tests/cr_tests/data/count_small.json").unwrap();
        let contents = read_to_string("../../tests/cr_tests/summary.html").unwrap();
        let template_info = TemplateInfo::Dynamic("../../example");
        let mut out: Vec<u8> = vec![];
        assert!(generate_html_summary(&json_data, contents, template_info, &mut out).is_ok());
        assert!(!out.is_empty());
    }
}
