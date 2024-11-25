//!
//! Defines structs backing various web summary components defined in
//! `websummary/src/components/**/*.rs`.
//!
//! The components defined in this module are summarized in the table. This
//! mapping is also implemented as a trait. See `ReactComponentName` below
//! and the implementations using the `react_component!` macro.
//!
//! | Struct Name | Component |
//! | ----------- | ----------- |
//! | HeroMetric | Metric.js |
//! | TitleWithTermDesc | DynamicHelptext.js |
//! | TitleWithHelp | HeaderWithHelp.js |
//! | GenericTable | Table.js |
//! | TableMetric | TableMetric.js |
//! | PlotlyChart | Plot.js |
//! | RawImage | RawImage.js |
//! | BlendedImage | ImageRegistViewer.js |
//! | VegaLitePlot | VegaLitePlot.js |
//! | Tooltip | ReactTooltip.js |
//!

use std::{collections::HashMap, fmt::Display, marker::PhantomData};

use anyhow::Error;
use itertools::Itertools;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{react_component, AddToSharedResource, HtmlTemplate, SharedResources};

impl HtmlTemplate for String {
    fn template(&self, _data_key: Option<String>) -> String {
        self.clone()
    }
}

fn join_data_key(data_key: &Option<String>, key: &str) -> String {
    match data_key {
        Some(prefix) => format!("{prefix}.{key}"),
        None => key.into(),
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// Threshold for the hero metric which determines the color
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Threshold {
    /// Pass will be shown in green color
    Pass,
    /// Warn will be shown in orange color
    Warn,
    /// Error will be shown in red color
    Error,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// This struct determines the navigation bar and header in the web summary.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsNavBar {
    /// Header after the 10x logo at the top
    pub pipeline: String,
    /// Page title is {id} - {description}
    pub id: String,
    /// Page title is {id} - {description}
    pub description: String,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// HeroMetric is a statistic that you want to highlight. You can optionally
/// control the color by choosing appropriate `threshold`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeroMetric {
    /// Name of the metric
    pub name: String,
    /// String formatted value of the metric
    pub metric: String,
    /// Optionally control the display color
    pub threshold: Option<Threshold>,
}

impl HeroMetric {
    pub fn new<N: Display, V: Display>(name: N, value: V) -> Self {
        HeroMetric {
            name: name.to_string(),
            metric: value.to_string(),
            threshold: None,
        }
    }
    pub fn with_threshold<N: Display, V: Display>(name: N, value: V, threshold: Threshold) -> Self {
        HeroMetric {
            name: name.to_string(),
            metric: value.to_string(),
            threshold: Some(threshold),
        }
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// Usually used to attach heading to a card with a help snippet
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TitleWithHelp {
    #[serde(rename = "helpText")]
    pub help: String,
    pub title: String,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// Description of a specific term or metric under the collapsible help.
/// First element of the tuple is the term and the second element is the
/// description. The term is shown in bold text.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TermDesc(pub String, pub Vec<String>);
impl TermDesc {
    pub fn with_one_desc(term: impl ToString, desc: impl ToString) -> Self {
        TermDesc(term.to_string(), vec![desc.to_string()])
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// Similar to `TitleWithHelp` but the help text will be a bunch of terms and
/// descriptions insteads of a single line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TitleWithTermDesc {
    pub title: String,
    pub data: Vec<TermDesc>,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// A single row in a table, which is simply a vector of String
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableRow(pub Vec<String>);

impl TableRow {
    pub fn two_col(c1: impl ToString, c2: impl ToString) -> Self {
        TableRow(vec![c1.to_string(), c2.to_string()])
    }
}

impl From<Vec<String>> for TableRow {
    fn from(item: Vec<String>) -> Self {
        TableRow(item)
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// Table with optional headers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GenericTable {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<Vec<String>>,
    pub rows: Vec<TableRow>,
}

impl GenericTable {
    /// Generate a generic table from rows and header
    pub fn from_rows(rows_vec: Vec<Vec<String>>, header: Option<Vec<String>>) -> Self {
        let rows = rows_vec.into_iter().map(TableRow::from).collect();
        GenericTable { header, rows }
    }

    /// Generate a generic table from columns
    /// Uses the headers in creating the GenericTable if provided
    pub fn from_columns(columns: Vec<Vec<String>>, header: Option<Vec<String>>) -> Self {
        let num_cols = columns.len();
        let num_rows = columns.iter().map(std::vec::Vec::len).max().unwrap();

        let mut rows = vec![vec![String::new(); num_cols]; num_rows];
        for (col_num, column) in columns.into_iter().enumerate() {
            for (row_num, val) in column.into_iter().enumerate() {
                rows[row_num][col_num] = val;
            }
        }

        GenericTable::from_rows(rows, header)
    }
}

fn deserialize_tuple_list_as_string<'de, D>(
    deserializer: D,
) -> Result<Vec<(String, String)>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Vec<(NumOrStr, NumOrStr)> = serde::de::Deserialize::deserialize(deserializer)?;
    Ok(value
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect())
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// A table containing two columns and no header, typically used to show a list
/// of metrics. The left column is the name and the right column is the value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableMetric {
    /// Vector of (metric name, metric value)
    #[serde(deserialize_with = "deserialize_tuple_list_as_string")]
    pub rows: Vec<(String, String)>,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// A plotly chart object. The `plotly` crate in rust provides a good API
/// for producing different types of plotly charts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PlotlyChart {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<Value>,
    pub data: Vec<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<Style>,
}

impl PlotlyChart {
    pub fn with_layout_and_data<L: Serialize, D: Serialize>(layout: L, data: Vec<D>) -> Self {
        PlotlyChart {
            config: Some(Self::default_config()),
            data: data
                .into_iter()
                .map(|d| serde_json::to_value(&d).unwrap())
                .collect(),
            layout: Some(serde_json::to_value(&layout).unwrap()),
            style: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    pub fn from_json_str(json_str: &str) -> Result<Self, Error> {
        Ok(serde_json::from_str(json_str)?)
    }

    pub fn default_config() -> Value {
        const DEFAULT_PLOTLY_CONFIG: &str = r#"{
            "displayModeBar": true,
            "staticPlot": false,
            "dragmode": "zoom",
            "modeBarButtons": [
                [
                    "toImage"
                ]
            ]
        }"#;
        serde_json::from_str::<Value>(DEFAULT_PLOTLY_CONFIG).unwrap()
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// A tooltip that appears on hover of the underlying `content`
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Tooltip {
    pub id: String,
    pub tooltip: String,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<TooltipVariant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place: Option<TooltipPlace>,
}

impl Tooltip {
    /// Create a tooltip with a generated ID. If you are creating multiple
    /// tooltips which will not be displayed simultaneously, you should prefer
    /// `Tooltip::new_with_id` and re-use the same ID.
    pub fn new(
        tooltip: impl Into<String>,
        content: impl Into<String>,
        variant: Option<TooltipVariant>,
        place: Option<TooltipPlace>,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let id = format!("tt-{}", rng.gen::<u16>());

        Self::new_with_id(id, tooltip, content, variant, place)
    }

    pub fn new_with_id(
        id: impl Into<String>,
        tooltip: impl Into<String>,
        content: impl Into<String>,
        variant: Option<TooltipVariant>,
        place: Option<TooltipPlace>,
    ) -> Self {
        Self {
            id: id.into(),
            tooltip: tooltip.into(),
            content: content.into(),
            variant,
            place,
        }
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// The tooltip variant
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TooltipVariant {
    #[default]
    Dark,
    Light,
    Success,
    Warning,
    Error,
    Info,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// The place to anchor a tooltip
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum TooltipPlace {
    #[default]
    Top,
    TopStart,
    TopEnd,
    Right,
    RightStart,
    RightEnd,
    Bottom,
    BottomStart,
    BottomEnd,
    Left,
    LeftStart,
    LeftEnd,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// Vega lite plot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct VegaLitePlot {
    pub spec: Value,
    pub actions: Option<Value>,
    #[serde(default)]
    pub renderer: Option<VegaLiteRenderer>,
}

impl VegaLitePlot {
    pub fn from_json_str(json_str: &str) -> Result<Self, Error> {
        Ok(VegaLitePlot {
            spec: serde_json::from_str(json_str)?,
            actions: None,
            renderer: None,
        })
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// The renderer to use for a Vega lite plot
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VegaLiteRenderer {
    #[default]
    Canvas,
    Svg,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// A CSS style definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Style(HashMap<String, String>);

impl Style {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn new() -> Style {
        Style(HashMap::new())
    }
    pub fn push(&mut self, key: impl Into<String>, val: impl Into<String>) {
        self.0.insert(key.into(), val.into());
    }
    pub fn set(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.push(key, val);
        self
    }
    pub fn width(mut self, val: impl Into<String>) -> Self {
        self.push("width", val);
        self
    }
    pub fn height(mut self, val: impl Into<String>) -> Self {
        self.push("height", val);
        self
    }
    pub fn pixelated(mut self) -> Self {
        self.push("image-rendering", "pixelated");
        self
    }
}

impl FromIterator<(String, String)> for Style {
    fn from_iter<I: IntoIterator<Item = (String, String)>>(iter: I) -> Self {
        let mut style = Style::new();
        for (key, val) in iter {
            style.push(key, val)
        }
        style
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinMax<T> {
    pub min: T,
    pub max: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialZoomPan {
    pub scale: Option<f64>,
    pub dx: Option<f64>,
    pub dy: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageZoomPan {
    scale_limits: MinMax<f64>,
    initial: Option<InitialZoomPan>,
    height: Option<u32>,
    width: Option<u32>,
}

impl ImageZoomPan {
    pub fn with_scale_limits(min_scale: f64, max_scale: f64) -> Self {
        ImageZoomPan {
            scale_limits: MinMax {
                min: min_scale,
                max: max_scale,
            },
            initial: None,
            height: None,
            width: None,
        }
    }
    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height);
        self
    }
    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width);
        self
    }
    pub fn initial(mut self, initial: InitialZoomPan) -> Self {
        self.initial = Some(initial);
        self
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ImageProps {
    width: Option<String>,
    height: Option<String>,
    #[serde(default)]
    style: Style,
}

impl ImageProps {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn width(mut self, width: impl ToString) -> Self {
        self.width = Some(width.to_string());
        self
    }
    pub fn height(mut self, height: impl ToString) -> Self {
        self.height = Some(height.to_string());
        self
    }
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
    /// Set `width = "100%"` in the img tag
    pub fn container_width(self) -> Self {
        self.width("100%")
    }
    pub fn pixelated(mut self) -> Self {
        self.style = self.style.pixelated();
        self
    }
    pub fn centered(mut self) -> Self {
        self.style = self
            .style
            .set("display", "block")
            .set("margin-left", "auto")
            .set("margin-right", "auto");
        self
    }
}

/// A raw image that needs to be encoded in base64
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawImage {
    /// Base 64 encoded image
    encoded_image: String,
    zoom_pan: Option<ImageZoomPan>,
    link: Option<String>,
    #[serde(flatten)]
    props: ImageProps,
}

impl RawImage {
    pub fn new(encoded_image: String) -> Self {
        RawImage {
            encoded_image,
            props: ImageProps::new(),
            zoom_pan: None,
            link: None,
        }
    }
    pub fn props(mut self, props: ImageProps) -> Self {
        self.props = props;
        self
    }
    pub fn pixelated(mut self) -> Self {
        self.props = self.props.pixelated();
        self
    }
    pub fn zoomable(mut self, min_scale: f64, max_scale: f64) -> Self {
        self.zoom_pan = Some(ImageZoomPan::with_scale_limits(min_scale, max_scale));
        self
    }
    pub fn with_link(mut self, link: &str) -> Self {
        self.link = Some(link.into());
        self
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropdownOption<T> {
    pub name: String,
    pub component: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CssAlign {
    #[default]
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DropdownSelectorProps {
    pub label: Option<String>,
    pub align: CssAlign,
}

/// Dropdown to toggle between different options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropdownSelector<T> {
    pub props: DropdownSelectorProps,
    pub options: Vec<DropdownOption<T>>,
}

impl<T: HtmlTemplate> HtmlTemplate for DropdownSelector<T> {
    fn template(&self, data_key: Option<String>) -> String {
        let base_data_key = join_data_key(&data_key, "options");
        let inner = self
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let this_inner = option
                    .component
                    .template(Some(format!("{base_data_key}[{i}].component")));
                format!(
                    r#"<div class="dropdown-wrapper" name="{}">{this_inner}</div>"#,
                    option.name
                )
            })
            .join("\n");
        let props_data_key = join_data_key(&data_key, "props");
        format!(
            r#"<div data-key="{props_data_key}" data-component="DropdownSelector">{inner}</div>"#
        )
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonSelectorOption<T> {
    pub name: String,
    pub component: T,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ButtonSelectoryType {
    #[default]
    FullWidth,
    Compact,
    Separated,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ButtonSelectorProps {
    #[serde(rename = "type")]
    pub ty: ButtonSelectoryType,
}

/// Button to toggle between different options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonSelector<T> {
    pub props: ButtonSelectorProps,
    pub options: Vec<ButtonSelectorOption<T>>,
}

impl<T: HtmlTemplate> HtmlTemplate for ButtonSelector<T> {
    fn template(&self, data_key: Option<String>) -> String {
        let base_data_key = join_data_key(&data_key, "options");
        let inner = self
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let this_inner = option
                    .component
                    .template(Some(format!("{base_data_key}[{i}].component")));
                format!(r#"<div name="{}">{this_inner}</div>"#, option.name)
            })
            .join("\n");
        let props_data_key = join_data_key(&data_key, "props");
        format!(r#"<div data-key="{props_data_key}" data-component="ButtonSelector">{inner}</div>"#)
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum NumOrStr {
    Num(usize),
    Str(String),
}

impl<'a> From<&'a str> for NumOrStr {
    fn from(value: &'a str) -> Self {
        NumOrStr::Str(value.into())
    }
}

impl From<usize> for NumOrStr {
    fn from(value: usize) -> Self {
        NumOrStr::Num(value)
    }
}

impl Display for NumOrStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NumOrStr::Num(n) => write!(f, "{}", n),
            NumOrStr::Str(s) => write!(f, "{}", s),
        }
    }
}

/// Controls the opacity slider width in a blended image
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlendedImageSliderSize {
    pub width: NumOrStr,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// Show two images on top of each other with a slider to adjust opacity.
/// Typically used to show two aligned images
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlendedImage {
    /// Base64 encoded image
    #[serde(rename = "imgA")]
    pub image1: String,
    /// Optional title that appears at the left of the opacity slider
    #[serde(rename = "imgATitle")]
    pub image1_title: Option<String>,
    /// Base64 encoded image
    #[serde(rename = "imgB")]
    pub image2: String,
    /// Optional title that appears at the right of the opacity slider
    #[serde(rename = "imgBTitle")]
    pub image2_title: Option<String>,
    #[serde(rename = "sizes")]
    pub size: BlendedImageSliderSize,
    pub plot_title: Option<String>,
    pub slider_title: Option<String>,
}

impl AddToSharedResource for BlendedImage {
    fn add_to_shared_resource(&mut self, resources: &mut SharedResources) {
        self.image1 = resources.insert(Value::String(self.image1.clone()));
        self.image2 = resources.insert(Value::String(self.image2.clone()));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlendedImageZoomable {
    #[serde(flatten)]
    blended_image: BlendedImage,
    #[serde(rename = "imgATransform")]
    image1_transform: Option<Vec<f64>>,
    #[serde(rename = "imgBTransform")]
    image2_transform: Option<Vec<f64>>,
    img_props: ImageProps,
    zoom_pan: ImageZoomPan,
    slider_top: Option<bool>,
}

impl AddToSharedResource for BlendedImageZoomable {
    fn add_to_shared_resource(&mut self, resources: &mut SharedResources) {
        self.blended_image.add_to_shared_resource(resources);
    }
}

impl BlendedImageZoomable {
    pub fn new(blended_image: BlendedImage, min_scale: f64, max_scale: f64) -> Self {
        BlendedImageZoomable {
            blended_image,
            img_props: ImageProps::new(),
            zoom_pan: ImageZoomPan::with_scale_limits(min_scale, max_scale),
            slider_top: None,
            image1_transform: None,
            image2_transform: None,
        }
    }
    pub fn zoom_pan_height(mut self, height: u32) -> Self {
        self.zoom_pan.height = Some(height);
        self
    }
    pub fn zoom_pan_width(mut self, width: u32) -> Self {
        self.zoom_pan.width = Some(width);
        self
    }
    pub fn zoom_pan_initial(mut self, initial: InitialZoomPan) -> Self {
        self.zoom_pan.initial = Some(initial);
        self
    }
    pub fn img_props(mut self, props: ImageProps) -> Self {
        self.img_props = props;
        self
    }
    pub fn slider_on_bottom(mut self) -> Self {
        self.slider_top = Some(false);
        self
    }
    pub fn slider_on_top(mut self) -> Self {
        self.slider_top = Some(true);
        self
    }
    pub fn image1_transform(mut self, transform: Vec<f64>) -> Self {
        self.image1_transform = Some(transform);
        self
    }
    pub fn image2_transform(mut self, transform: Vec<f64>) -> Self {
        self.image2_transform = Some(transform);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomViewerSize {
    pub width: NumOrStr,
    pub height: NumOrStr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoomViewer {
    pub small_image: String,
    pub big_image: String,
    pub sizes: ZoomViewerSize,
    pub plot_title: Option<String>,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// The react component name corresponsing to the object. By knowing the
/// component name of the object, we know how to write html for the object.
pub trait ReactComponent {
    fn component_name() -> &'static str;
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// The mapping between structs defined in this module and the react components
// defined in `websummary/src/components/**/*.js`.
react_component!(HeroMetric, "Metric");
react_component!(TitleWithTermDesc, "DynamicHelptext");
react_component!(TitleWithHelp, "HeaderWithHelp");
react_component!(GenericTable, "Table");
react_component!(TableMetric, "TableMetric");
react_component!(PlotlyChart, "Plot");
react_component!(VegaLitePlot, "VegaLitePlot");
react_component!(RawImage, "RawImage");
react_component!(BlendedImage, "ImageRegistViewer");
react_component!(BlendedImageZoomable, "BlenderViewerZoomable");
react_component!(ZoomViewer, "ZoomViewer");
react_component!(StepProgress, "StepProgress");
react_component!(CodeBlock, "CodeBlock");
react_component!(Tooltip, "ReactTooltip");
react_component!(HdClusteringPlot, "HdClusteringPlot");
react_component!(HtmlFragment, "HtmlFragment");
react_component!(JavaScript, "JavaScript");
react_component!(DifferentialExpressionTable, "DifferentialExpressionTable");
react_component!(HdEndToEndAlignment, "HdEndToEndAlignment");
react_component!(MultiLayerImages, "MultiLayerImages");
react_component!(DownloadableFile, "DownloadableFile");

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
impl<T: ReactComponent> HtmlTemplate for T {
    fn template(&self, data_key: Option<String>) -> String {
        let data_key = data_key.unwrap_or_else(|| {
            panic!(
                "data-key is required to convert a react component {} into a template",
                T::component_name()
            )
        });
        format!(
            r#"<div data-key="{data_key}" data-component="{}"></div>"#,
            T::component_name()
        )
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Show progress in a series of steps
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepProgress {
    pub steps: Vec<String>,
    pub active_step: u8,
    pub active_step_failed: bool,
}

pub trait ParentComponentProps {
    fn parent_component_name() -> &'static str;
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// A wrapper component that has both props and children
#[derive(Serialize, Deserialize, Clone)]
pub struct ComponentWithChildren<P: ParentComponentProps, C: HtmlTemplate> {
    pub parent_props: P,
    pub children: C,
}

impl<P: ParentComponentProps, C: HtmlTemplate> ComponentWithChildren<P, C> {
    pub fn new(parent_props: P, children: C) -> Self {
        ComponentWithChildren {
            parent_props,
            children,
        }
    }
}

impl<P: ParentComponentProps, C: HtmlTemplate> HtmlTemplate for ComponentWithChildren<P, C> {
    fn template(&self, data_key: Option<String>) -> String {
        const PARENT_PROPS: &str = "parent_props";
        const CHILDREN: &str = "children";
        let (component_key, children_key) = match data_key {
            Some(key) => (format!("{key}.{PARENT_PROPS}"), format!("{key}.{CHILDREN}")),
            None => (PARENT_PROPS.into(), CHILDREN.into()),
        };
        let children = self.children.template(Some(children_key));
        let component_name = P::parent_component_name();
        format!(
            r#"<div data-key="{component_key}" data-component="{component_name}">
{children}
</div>"#
        )
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Inline alerts which can show up anywhere in the html unlike a top level alert
#[derive(Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum InlineAlertLevel {
    Primary,
    Secondary,
    Success,
    Danger,
    Warning,
    Info,
    Light,
    Dark,
}

#[derive(Serialize, Clone)]
pub struct InlineAlertProps {
    pub level: InlineAlertLevel,
}

impl ParentComponentProps for InlineAlertProps {
    fn parent_component_name() -> &'static str {
        "InlineAlert"
    }
}

pub type InlineAlert<T> = ComponentWithChildren<InlineAlertProps, T>;
pub type InlineTextAlert = InlineAlert<HtmlFragment>;

impl InlineTextAlert {
    pub fn with_level_and_text(level: InlineAlertLevel, text: impl ToString) -> Self {
        InlineAlert::new(InlineAlertProps { level }, HtmlFragment::new(text))
    }
    pub fn primary(text: impl ToString) -> Self {
        InlineTextAlert::with_level_and_text(InlineAlertLevel::Primary, text)
    }
    pub fn secondary(text: impl ToString) -> Self {
        InlineTextAlert::with_level_and_text(InlineAlertLevel::Secondary, text)
    }
    pub fn success(text: impl ToString) -> Self {
        InlineTextAlert::with_level_and_text(InlineAlertLevel::Success, text)
    }
    pub fn danger(text: impl ToString) -> Self {
        InlineTextAlert::with_level_and_text(InlineAlertLevel::Danger, text)
    }
    pub fn warning(text: impl ToString) -> Self {
        InlineTextAlert::with_level_and_text(InlineAlertLevel::Warning, text)
    }
    pub fn info(text: impl ToString) -> Self {
        InlineTextAlert::with_level_and_text(InlineAlertLevel::Info, text)
    }
    pub fn light(text: impl ToString) -> Self {
        InlineTextAlert::with_level_and_text(InlineAlertLevel::Light, text)
    }
    pub fn dark(text: impl ToString) -> Self {
        InlineTextAlert::with_level_and_text(InlineAlertLevel::Dark, text)
    }
}

#[derive(Serialize, Clone)]
pub struct InlineHelpProps;

impl ParentComponentProps for InlineHelpProps {
    fn parent_component_name() -> &'static str {
        "InlineHelp"
    }
}

pub type InlineHelp = ComponentWithChildren<InlineHelpProps, HtmlFragment>;

impl InlineHelp {
    pub fn with_content(html: String) -> Self {
        InlineHelp::new(InlineHelpProps, HtmlFragment::new(html))
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// HTML heading

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum HeadingLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl HeadingLevel {
    fn tag(self) -> &'static str {
        match self {
            HeadingLevel::H1 => "h1",
            HeadingLevel::H2 => "h2",
            HeadingLevel::H3 => "h3",
            HeadingLevel::H4 => "h4",
            HeadingLevel::H5 => "h5",
            HeadingLevel::H6 => "h6",
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Heading {
    text: String,
    level: HeadingLevel,
}

impl Heading {
    pub fn new(level: HeadingLevel, text: impl ToString) -> Self {
        Heading {
            text: text.to_string(),
            level,
        }
    }
    pub fn h1(text: impl ToString) -> Self {
        Heading::new(HeadingLevel::H1, text)
    }
    pub fn h2(text: impl ToString) -> Self {
        Heading::new(HeadingLevel::H2, text)
    }
    pub fn h3(text: impl ToString) -> Self {
        Heading::new(HeadingLevel::H3, text)
    }
    pub fn h4(text: impl ToString) -> Self {
        Heading::new(HeadingLevel::H4, text)
    }
    pub fn h5(text: impl ToString) -> Self {
        Heading::new(HeadingLevel::H5, text)
    }
    pub fn h6(text: impl ToString) -> Self {
        Heading::new(HeadingLevel::H6, text)
    }
}

impl HtmlTemplate for Heading {
    fn template(&self, _: Option<String>) -> String {
        let tag = self.level.tag();
        format!("<{tag}>{}</{tag}>", self.text)
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// Two column

#[cfg(feature = "derive")]
#[derive(Debug, Clone, Serialize, Deserialize, tenx_websummary_derive::HtmlTemplate)]
#[html(websummary_crate = "crate")]
pub struct TwoColumn<L: HtmlTemplate, R: HtmlTemplate> {
    #[html(row = "1")]
    pub left: L,
    #[html(row = "1")]
    pub right: R,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Collapsible panel
#[derive(Serialize, Deserialize)]
pub struct CollapsablePanelProps {
    pub title: String,
    pub plain: Option<bool>,
}

impl ParentComponentProps for CollapsablePanelProps {
    fn parent_component_name() -> &'static str {
        "CollapsablePanel"
    }
}

pub type CollapsablePanel<T> = ComponentWithChildren<CollapsablePanelProps, T>;

impl<T: HtmlTemplate> CollapsablePanel<T> {
    pub fn with_title_and_content(title: impl ToString, content: T) -> Self {
        CollapsablePanel::new(
            CollapsablePanelProps {
                title: title.to_string(),
                plain: Some(false),
            },
            content,
        )
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Block of preformatted text block
#[derive(Serialize, Deserialize)]
pub struct CodeBlock {
    pub code: String,
    #[serde(rename = "maxHeight")]
    pub max_height: Option<String>,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Encapsulate all flavours of titles in an enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Title {
    WithHelp(TitleWithHelp),
    WithTermDesc(TitleWithTermDesc),
}

impl Title {
    pub fn new(title: impl Into<String>) -> Self {
        Title::WithHelp(TitleWithHelp {
            help: String::new(),
            title: title.into(),
        })
    }
}

impl From<TitleWithHelp> for Title {
    fn from(src: TitleWithHelp) -> Self {
        Title::WithHelp(src)
    }
}

impl From<TitleWithTermDesc> for Title {
    fn from(src: TitleWithTermDesc) -> Self {
        Title::WithTermDesc(src)
    }
}

impl HtmlTemplate for Title {
    fn template(&self, data_key: Option<String>) -> String {
        match self {
            Title::WithHelp(t) => t.template(data_key),
            Title::WithTermDesc(t) => t.template(data_key),
        }
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// An element with a title
#[cfg(feature = "derive")]
#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Eq, tenx_websummary_derive::HtmlTemplate,
)]
#[html(websummary_crate = "crate")]
pub struct WithTitle<T: HtmlTemplate> {
    pub title: Title,
    pub inner: T,
}

#[cfg(feature = "derive")]
impl<T: HtmlTemplate> WithTitle<T> {
    pub fn new(title: Title, inner: T) -> Self {
        Self { title, inner }
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// String holding javascript code
#[derive(Debug, Serialize, Clone)]
pub struct JavaScript {
    pub code: String,
}

impl JavaScript {
    pub fn new(code: impl ToString) -> Self {
        JavaScript {
            code: code.to_string(),
        }
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// String holding html
#[derive(Debug, Serialize, Clone)]
pub struct HtmlFragment {
    pub html: String,
}

impl HtmlFragment {
    pub fn new(html: impl ToString) -> Self {
        HtmlFragment {
            html: html.to_string(),
        }
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// Wrapping underlying template within a div. Useful for layout customization
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(transparent)] // Works becasue we skip `class`
struct DivWrapper<'a, T: HtmlTemplate> {
    inner: &'a T,
    #[serde(skip)]
    class: String,
}

impl<'a, T: HtmlTemplate> DivWrapper<'a, T> {
    fn new(inner: &'a T, class: &str) -> Self {
        DivWrapper {
            inner,
            class: class.to_string(),
        }
    }
    fn row(inner: &'a T) -> Self {
        DivWrapper::new(inner, "row")
    }
    fn col(inner: &'a T) -> Self {
        DivWrapper::new(inner, "col")
    }
}

impl<'a, T: HtmlTemplate> HtmlTemplate for DivWrapper<'a, T> {
    fn template(&self, data_key: Option<String>) -> String {
        format!(
            "<div class=\"{}\">\n{}\n</div>",
            self.class,
            self.inner.template(data_key)
        )
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// Layout of the grid
#[derive(Debug, Clone)]
pub enum GridLayout {
    // A grid with upto a given number of columns and a responsive layout
    MaxCols(u8),
    // A grid with upto a given number of columns but non-responsive layout
    MaxColsNonResponsive(u8),
}

impl GridLayout {
    fn col_class(&self) -> &'static str {
        match self {
            GridLayout::MaxCols(ncols) => match *ncols {
                2 => "col-sm-6",
                3 => "col-sm-4",
                4 => "col-sm-3",
                6 => "col-sm-2",
                _ => "col",
            },
            _ => unimplemented!(),
        }
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// A grid of elements all of the same type. This is a this wrapper around DynGrid
/// which can hold elements of different types in a grid.
#[derive(Serialize)]
pub struct Grid<T: HtmlTemplate> {
    #[serde(skip)]
    phantom: PhantomData<T>,
    #[serde(flatten)]
    dyn_grid: DynGrid,
}

impl<T: 'static + HtmlTemplate + Serialize> Grid<T> {
    pub fn new(layout: GridLayout) -> Self {
        Grid {
            phantom: PhantomData,
            dyn_grid: DynGrid::new(layout),
        }
    }
    pub fn push(&mut self, element: T) {
        self.dyn_grid.push(element)
    }
    pub fn with_elements(elements: Vec<T>, layout: GridLayout) -> Self {
        let mut grid = Grid::new(layout);
        for element in elements {
            grid.push(element);
        }
        grid
    }
}

impl<T: HtmlTemplate> HtmlTemplate for Grid<T> {
    fn template(&self, data_key: Option<String>) -> String {
        self.dyn_grid.template(data_key)
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// A unique marker for keys in the DynGrid divs. This will be replaced
// with the correct key when building the template.
const DYN_GRID_MARKER: &str = "__AUbkUE__DYN_GRID__WhcSw=__";

/// A grid that can hold elements of different types.
#[derive(Serialize, Clone)]
pub struct DynGrid {
    grid_data: Vec<Value>,
    #[serde(skip)]
    elements: Vec<String>,
    #[serde(skip)]
    layout: GridLayout,
}

impl DynGrid {
    pub fn new(layout: GridLayout) -> Self {
        DynGrid {
            grid_data: vec![],
            elements: vec![],
            layout,
        }
    }
    pub fn push<T: HtmlTemplate + Serialize>(&mut self, element: T) {
        self.grid_data.push(serde_json::to_value(&element).unwrap());
        self.elements
            .push(element.template(Some(DYN_GRID_MARKER.into())));
    }
    pub fn with_elements<T: 'static + HtmlTemplate + Serialize>(
        elements: Vec<T>,
        layout: GridLayout,
    ) -> Self {
        let mut grid = DynGrid::new(layout);
        for element in elements {
            grid.push(element);
        }
        grid
    }
}

impl HtmlTemplate for DynGrid {
    fn template(&self, data_key: Option<String>) -> String {
        match self.layout {
            GridLayout::MaxCols(n) => self
                .elements
                .iter()
                .enumerate()
                .chunks(n as usize)
                .into_iter()
                .map(|same_row_elements| {
                    DivWrapper::row(
                        &same_row_elements
                            .map(|(i, element)| {
                                let data_key = match &data_key {
                                    Some(key) => format!("{key}.grid_data[{i}]"),
                                    None => format!("grid_data[{i}]"),
                                };
                                DivWrapper::new(
                                    &element.replace(DYN_GRID_MARKER, &data_key),
                                    self.layout.col_class(),
                                )
                                .template(None)
                            })
                            .join("\n"),
                    )
                    .template(None)
                })
                .join("\n"),
            GridLayout::MaxColsNonResponsive(n) => {
                let rows = self
                    .elements
                    .iter()
                    .enumerate()
                    .chunks(n as usize)
                    .into_iter()
                    .map(|same_row_elements| {
                        let tds = same_row_elements
                            .map(|(i, element)| {
                                let data_key = match &data_key {
                                    Some(key) => format!("{key}.grid_data[{i}]"),
                                    None => format!("grid_data[{i}]"),
                                };
                                format!("<td>{}</td>", element.replace(DYN_GRID_MARKER, &data_key))
                            })
                            .join("\n");
                        format!("<tr>{}</tr>", tds)
                    })
                    .join("\n");
                format!("<table><tbody>{}</tbody></table>", rows)
            }
        }
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// A card which has a raised border
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Card<T: HtmlTemplate> {
    #[serde(flatten)]
    inner: T,
    #[serde(skip, default)]
    width: CardWidth,
}

impl<T: HtmlTemplate> Card<T> {
    pub fn half_width(inner: T) -> Self {
        Card {
            inner,
            width: CardWidth::Half,
        }
    }
    pub fn with_width(inner: T, width: CardWidth) -> Self {
        Card { inner, width }
    }
    pub fn full_width(inner: T) -> Self {
        Card {
            inner,
            width: CardWidth::Full,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CardWidth {
    Full,
    #[default]
    Half,
}

impl CardWidth {
    fn class(&self) -> &'static str {
        match self {
            CardWidth::Full => "summary_row",
            CardWidth::Half => "summary_card",
        }
    }
}

impl<T: HtmlTemplate> HtmlTemplate for Card<T> {
    fn template(&self, data_key: Option<String>) -> String {
        DivWrapper::new(&self.inner, self.width.class()).template(data_key)
    }
}

impl<T: HtmlTemplate> HtmlTemplate for Option<T> {
    fn template(&self, data_key: Option<String>) -> String {
        self.as_ref()
            .map(|inner| inner.template(data_key))
            .unwrap_or_default()
    }
}

impl<T: HtmlTemplate> HtmlTemplate for Vec<T> {
    fn template(&self, data_key: Option<String>) -> String {
        self.iter()
            .enumerate()
            .map(|(i, inner)| {
                DivWrapper::row(&DivWrapper::col(inner))
                    .template(data_key.as_ref().map(|k| format!("{k}[{i}]")))
            })
            .join("\n")
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// A unique marker for keys in the Tabs divs. This will be replaced
// with the correct key when building the template.
const TAB_MARKER: &str = "__AUbkUE__TAB__WhcSw=__";

/// Each tab is defined by a title and an element
/// TODO: Support deriving tabs from a struct
#[derive(Serialize, Default)]
pub struct Tabs {
    tab_data: Vec<Value>,
    #[serde(skip)]
    elements: Vec<String>,
    #[serde(skip)]
    titles: Vec<String>,
}

impl Tabs {
    pub fn new() -> Self {
        Tabs::default()
    }
    pub fn push<T: HtmlTemplate + Serialize>(&mut self, tab_title: impl Into<String>, element: T) {
        self.tab_data.push(serde_json::to_value(&element).unwrap());
        self.elements
            .push(element.template(Some(TAB_MARKER.into())));
        self.titles.push(tab_title.into());
    }
    pub fn tab<T: HtmlTemplate + Serialize>(
        mut self,
        tab_title: impl Into<String>,
        element: T,
    ) -> Self {
        self.push(tab_title, element);
        self
    }
}

impl HtmlTemplate for Tabs {
    fn template(&self, data_key: Option<String>) -> String {
        let base_data_key = match data_key {
            Some(key) => format!("{key}.tab_data"),
            None => "tab_data".into(),
        };
        let inner = std::iter::zip(&self.elements, &self.titles)
            .enumerate()
            .map(|(i, (element, title))| {
                let inner = element.replace(TAB_MARKER, &format!("{base_data_key}[{i}]"));
                format!(
                    r#"<div class="tab-wrapper" data-event-key="tab_{i}" data-title="{title}">
{inner}
</div>"#
                )
            })
            .join("\n");
        format!(
            r#"<div class="tabs-wrapper" data-default-active-key="tab_0" data-id="main-tabs">
{inner}
</div>"#
        )
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// A text with a hyperlink
pub struct LinkedText {
    pub link: String,
    pub text: String,
}

impl LinkedText {
    pub fn html(&self) -> String {
        format!("<a href=\"{}\">{}</a>", self.link, self.text)
    }
}

impl HtmlTemplate for LinkedText {
    fn template(&self, _: Option<String>) -> String {
        self.html()
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// HdClusteringPlot

#[derive(Serialize, Deserialize)]
pub struct HdClusteringSingleClusterData {
    pub cluster_name: String,
    pub hex_color: String,
    pub spatial_plot: String,
    pub umap_plot: String,
}

#[derive(Serialize, Deserialize)]
pub struct HdClusteringSpatialPlotProps {
    pub title: String,
    pub tissue_image: String,
    pub tissue_css_transform: Vec<f64>,
    pub spot_css_transform: Vec<f64>,
    pub width: u32,
    pub height: u32,
    pub initial_zoom_pan: InitialZoomPan,
}

impl AddToSharedResource for HdClusteringSpatialPlotProps {
    fn add_to_shared_resource(&mut self, shared_resource: &mut SharedResources) {
        self.tissue_image = shared_resource.insert(Value::String(self.tissue_image.clone()));
    }
}

#[derive(Serialize, Deserialize)]
pub struct HdClusteringUmapPlotProps {
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct HdClusteringPlot {
    pub spatial_plot_props: HdClusteringSpatialPlotProps,
    pub umap_plot_props: HdClusteringUmapPlotProps,
    pub clusters: Vec<HdClusteringSingleClusterData>,
}

impl AddToSharedResource for HdClusteringPlot {
    fn add_to_shared_resource(&mut self, shared_resource: &mut SharedResources) {
        self.spatial_plot_props
            .add_to_shared_resource(shared_resource);
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DifferentialExpressionTable {
    pub table: Value,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HdEndToEndAlignment {
    pub tissue_image: String,
    pub tissue_image_title: String,
    pub tissue_css_transform: Vec<f64>,
    pub display_height: u32,
    pub display_width: u32,
    pub umi_legend_images: Vec<HdEndToEndAlignmentUmiLegendImage>,
    pub grayscale_umi_image: String,
    pub umi_image_title: String,
    pub umi_css_transform: Vec<f64>,
    pub tissue_mask_image: String,
    pub initial_zoom_pan: Option<InitialZoomPan>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HdEndToEndAlignmentUmiLegendImage {
    pub colormap: String,
    pub legend_image: String,
}

impl AddToSharedResource for HdEndToEndAlignmentUmiLegendImage {
    fn add_to_shared_resource(&mut self, shared_resource: &mut SharedResources) {
        self.legend_image = shared_resource.insert(Value::String(self.legend_image.clone()));
    }
}

impl AddToSharedResource for HdEndToEndAlignment {
    fn add_to_shared_resource(&mut self, shared_resource: &mut SharedResources) {
        self.tissue_image = shared_resource.insert(Value::String(self.tissue_image.clone()));
        self.grayscale_umi_image =
            shared_resource.insert(Value::String(self.grayscale_umi_image.clone()));
        for umi_image in &mut self.umi_legend_images {
            umi_image.add_to_shared_resource(shared_resource);
        }
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// MultiLayerImages
#[derive(Debug, Serialize, Deserialize)]
pub struct InitialFocus {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabeledImage {
    pub label: Option<String>,
    pub color: Option<String>,
    pub image: String,
    pub css_transform: Option<Vec<f64>>,
}

impl AddToSharedResource for LabeledImage {
    fn add_to_shared_resource(&mut self, shared_resource: &mut SharedResources) {
        self.image = shared_resource.insert(Value::String(self.image.clone()));
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub images: Vec<LabeledImage>,
}

impl AddToSharedResource for Layer {
    fn add_to_shared_resource(&mut self, shared_resource: &mut SharedResources) {
        self.images
            .iter_mut()
            .for_each(|labelled_image| labelled_image.add_to_shared_resource(shared_resource));
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiLayerImages {
    pub focus: InitialFocus,
    pub layers: Vec<Layer>,
    pub full_screen: bool,
}

impl AddToSharedResource for MultiLayerImages {
    fn add_to_shared_resource(&mut self, shared_resource: &mut SharedResources) {
        self.layers
            .iter_mut()
            .for_each(|layer| layer.add_to_shared_resource(shared_resource));
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Csv download link
#[derive(Serialize, Deserialize)]
pub struct DownloadableFile {
    pub data: String,
    pub filename: String,
    pub text: String,
    pub mime_type: String,
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// Command line template
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CommandLine {
    pub title: String,
    pub data: Vec<TermDesc>,
    pub show_dark_button_icon: bool,
}

impl ReactComponent for CommandLine {
    fn component_name() -> &'static str {
        "DynamicHelptext"
    }
}

impl CommandLine {
    pub fn new(cmdline: &str) -> Result<Self, Error> {
        Ok(Self {
            title: "Command Line Arguments".to_string(),
            data: vec![TermDesc("".to_string(), 
                vec![format!("<span style='font-size: 18px;'><code><pre style='white-space: pre-wrap;'>{}</pre></code></span>", 
                    cmdline)])],
            show_dark_button_icon: true,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    fn check_eq_json(j1: &str, j2: &str) {
        assert_eq!(
            serde_json::from_str::<serde_json::value::Value>(j1).unwrap(),
            serde_json::from_str::<serde_json::value::Value>(j2).unwrap()
        );
    }

    fn test_json_roundtrip<T: Serialize + serde::de::DeserializeOwned>(json: &str) -> T {
        let parsed: T = serde_json::from_str(json).unwrap();
        let parsed_str = serde_json::to_string(&parsed).unwrap();
        check_eq_json(&parsed_str, json);
        parsed
    }

    #[test]
    fn test_header_with_help() {
        test_json_roundtrip::<TitleWithHelp>(
            r#"{
            "helpText": "This is the help text", 
            "title": "This is the title"
        }"#,
        );
    }

    #[test]
    fn test_term_descriptions() {
        test_json_roundtrip::<TermDesc>(
            r#"[
                "Inner title", 
                [
                    "Inner help description 1",
                    "Inner help description 2"
                ]
            ]"#,
        );
    }

    #[test]
    fn test_dyn_help() {
        test_json_roundtrip::<TitleWithTermDesc>(
            r#"{
                "data": [
                    [
                        "Metric 1", 
                        [
                            "Help 1"
                        ]
                    ], 
                    [
                        "Metric 2", 
                        [
                            "Help 2"
                        ]
                    ]
                ], 
                "title": "Title text"
            }"#,
        );
    }

    #[test]
    fn test_config_valid_json() {
        let _ = PlotlyChart::default_config();
    }

    #[test]
    fn test_generic_table() {
        test_json_roundtrip::<GenericTable>(
            r#"{
                "rows": [
                    [
                        "Sample ID", 
                        "Human PBMC BCR"
                    ], 
                    [
                        "Sample Description", 
                        "Pre vs post vaccination"
                    ]
                ]
            }
            "#,
        );
    }

    #[test]
    fn test_generic_table_with_header() {
        test_json_roundtrip::<GenericTable>(
            r#"{
                "header": [
                    "Donor", 
                    "Origin", 
                    "Cells",
                    "Clonotypes"
                ], 
                "rows": [
                    [
                        "Donor1",
                        "PreVac",
                        "10,000",
                        "7,000"
                    ],
                    [
                        "Donor1",
                        "PostVac",
                        "8,000",
                        "2,000"
                    ]
                ]
            }"#,
        );
    }

    #[test]
    fn test_gentable_transpose() {
        let table_json = r#"{"header":["Donor","Origin","Cells","Clonotypes"],"rows":[["Donor1","PreVac","10,000","7,000"],["Donor2","","8,000","2,000"]]}"#;
        let header = Some(
            vec!["Donor", "Origin", "Cells", "Clonotypes"]
                .into_iter()
                .map(std::string::ToString::to_string)
                .collect(),
        );
        let columns_in = vec![
            vec!["Donor1".to_string(), "Donor2".to_string()],
            vec!["PreVac".to_string()],
            vec!["10,000".to_string(), "8,000".to_string()],
            vec!["7,000".to_string(), "2,000".to_string()],
        ];
        assert_eq!(
            table_json,
            serde_json::to_string(&GenericTable::from_columns(columns_in, header)).unwrap()
        );
    }

    #[test]
    fn test_deserialize_blended_image_width_number() {
        let json_str = r#"{
            "imgA": "data:image/jpg;base64,/9j/4AAQSkZJRgABAQAAAQABAA",
            "imgATitle": "CytAssist Image",
            "imgB": "data:image/jpg;base64,/9j/4AAQSkZJRgABAQAAAQABAA",
            "imgBTitle": "Microscope Image",
            "plot_title": "CytAssist Image Alignment",
            "sizes": {
                "width": 470
            },
            "slider_title": ""
        }"#;
        let _: BlendedImage = serde_json::from_str(json_str).unwrap();
    }

    #[test]
    fn test_deserialize_blended_image_width_string() {
        let json_str = r#"{
            "imgA": "data:image/jpg;base64,/9j/4AAQSkZJRgABAQAAAQABAA",
            "imgATitle": "CytAssist Image",
            "imgB": "data:image/jpg;base64,/9j/4AAQSkZJRgABAQAAAQABAA",
            "imgBTitle": "Microscope Image",
            "plot_title": "CytAssist Image Alignment",
            "sizes": {
                "width": "470px"
            },
            "slider_title": ""
        }"#;
        let _: BlendedImage = serde_json::from_str(json_str).unwrap();
    }
}
