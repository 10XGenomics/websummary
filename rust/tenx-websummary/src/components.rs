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
//!

use std::{collections::HashMap, fmt::Display, marker::PhantomData};

use anyhow::Error;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{react_component, HtmlTemplate};

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

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
/// A table containing two columns and no header, typically used to show a list
/// of metrics. The left column is the name and the right column is the value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TableMetric {
    /// Vector of (metric name, metric value)
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
/// Vega lite plot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct VegaLitePlot {
    pub spec: Value,
    pub actions: Option<Value>,
    #[serde(default)]
    pub renderer: Option<VegaLiteRenderer>,
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
pub struct ImageZoomPan {
    scale_limits: MinMax<f64>,
}

impl ImageZoomPan {
    pub fn with_scale_limits(min_scale: f64, max_scale: f64) -> Self {
        ImageZoomPan {
            scale_limits: MinMax {
                min: min_scale,
                max: max_scale,
            },
        }
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
    #[serde(flatten)]
    props: ImageProps,
}

impl RawImage {
    pub fn new(encoded_image: String) -> Self {
        RawImage {
            encoded_image,
            props: ImageProps::new(),
            zoom_pan: None,
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
        self.zoom_pan = Some(ImageZoomPan {
            scale_limits: MinMax {
                min: min_scale,
                max: max_scale,
            },
        });
        self
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropdownOption<T> {
    pub name: String,
    pub component: T,
}
/// Dropdown to toggle between different options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropdownSelector<T> {
    pub options: Vec<DropdownOption<T>>,
}

impl<T: HtmlTemplate> HtmlTemplate for DropdownSelector<T> {
    fn template(&self, data_key: Option<String>) -> String {
        let base_data_key = match data_key {
            Some(key) => format!("{key}.options"),
            None => "options".into(),
        };
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
        format!(r#"<div data-component="DropdownSelector">{inner}</div>"#)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlendedImageZoomable {
    #[serde(flatten)]
    blended_image: BlendedImage,
    img_props: ImageProps,
    zoom_pan: ImageZoomPan,
}

impl BlendedImageZoomable {
    pub fn new(blended_image: BlendedImage, min_scale: f64, max_scale: f64) -> Self {
        BlendedImageZoomable {
            blended_image,
            img_props: ImageProps::new(),
            zoom_pan: ImageZoomPan::with_scale_limits(min_scale, max_scale),
        }
    }
    pub fn img_props(mut self, props: ImageProps) -> Self {
        self.img_props = props;
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

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
impl<T: ReactComponent> HtmlTemplate for T {
    fn template(&self, data_key: Option<String>) -> String {
        let data_key =
            data_key.expect("data-key is required to convert a react component into a template");
        format!(
            r#"<div data-key="{data_key}" data-component="{}"></div>"#,
            T::component_name()
        )
    }
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
/// String holding html
#[derive(Debug, Serialize, Clone)]
struct HtmlFragment(String);
impl HtmlTemplate for HtmlFragment {
    fn template(&self, _: Option<String>) -> String {
        self.0.clone()
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
    // A grid with upto a given number of columns
    MaxCols(u8),
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
    elements: Vec<HtmlFragment>,
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
            .push(HtmlFragment(element.template(Some(DYN_GRID_MARKER.into()))));
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
        let col_class = self.layout.col_class();
        match self.layout {
            GridLayout::MaxCols(n) => self
                .elements
                .iter()
                .enumerate()
                .chunks(n as usize)
                .into_iter()
                .map(|same_row_elements| {
                    DivWrapper::row(&HtmlFragment(
                        same_row_elements
                            .map(|(i, element)| {
                                let data_key = match &data_key {
                                    Some(key) => format!("{key}.grid_data[{i}]"),
                                    None => format!("grid_data[{i}]"),
                                };
                                DivWrapper::new(
                                    &HtmlFragment(element.0.replace(DYN_GRID_MARKER, &data_key)),
                                    col_class,
                                )
                                .template(None)
                            })
                            .join("\n"),
                    ))
                    .template(None)
                })
                .join("\n"),
        }
    }
}

// :::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
// A card which has a raised border
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Card<T: HtmlTemplate>(pub T);

impl<T: HtmlTemplate> HtmlTemplate for Card<T> {
    fn template(&self, data_key: Option<String>) -> String {
        DivWrapper::new(&self.0, "summary_card").template(data_key)
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
    elements: Vec<HtmlFragment>,
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
            .push(HtmlFragment(element.template(Some(TAB_MARKER.into()))));
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
            .map(|(i, (HtmlFragment(ref element), title))| {
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
