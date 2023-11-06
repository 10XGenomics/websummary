//! Tests for websummary derive
#![cfg(feature = "derive")]

use pretty_assertions::assert_eq;
use serde::Serialize;
use tenx_websummary::components::{DynGrid, Grid, GridLayout, HeroMetric, RawImage};
use tenx_websummary::HtmlTemplate;

#[test]
fn test_html_template_simple() {
    #[derive(Serialize, Clone, HtmlTemplate)]
    struct WebSummaryContent {
        #[html(row = "1")]
        num_cells: HeroMetric,
        #[html(row = "1")]
        umis_per_cell: HeroMetric,
        valid_bc_read_frac: HeroMetric,
    }
    const EXPECTED_TEMPLATE: &str = r#"<div class="row">
<div class="col">
<div data-key="num_cells" data-component="Metric"></div>
</div>
<div class="col">
<div data-key="umis_per_cell" data-component="Metric"></div>
</div>
</div>
<div class="row">
<div class="col">
<div data-key="valid_bc_read_frac" data-component="Metric"></div>
</div>
</div>
"#;
    let content = WebSummaryContent {
        num_cells: HeroMetric::new("Number of cells", "3,487"),
        umis_per_cell: HeroMetric::new("Median UMIs per cell", "867"),
        valid_bc_read_frac: HeroMetric::new("Valid barcodes", "93.6%"),
    };
    assert_eq!(EXPECTED_TEMPLATE, content.template(None));
}

#[test]
fn test_html_template_nested() {
    #[derive(Serialize, Clone, HtmlTemplate)]
    struct LeftContent {
        #[html(row = "1")]
        num_cells: HeroMetric,
        #[html(row = "1")]
        umis_per_cell: HeroMetric,
    }

    #[derive(Serialize, Clone, HtmlTemplate)]
    struct FullContent {
        #[html(row = "1")]
        left: LeftContent,
        #[html(row = "1")]
        valid_bc_read_frac: HeroMetric,
    }

    const EXPECTED_TEMPLATE: &str = r#"<div class="row">
<div class="col">
<div class="row">
<div class="col">
<div data-key="left.num_cells" data-component="Metric"></div>
</div>
<div class="col">
<div data-key="left.umis_per_cell" data-component="Metric"></div>
</div>
</div>

</div>
<div class="col">
<div data-key="valid_bc_read_frac" data-component="Metric"></div>
</div>
</div>
"#;
    let content = FullContent {
        left: LeftContent {
            num_cells: HeroMetric::new("Number of cells", "3,487"),
            umis_per_cell: HeroMetric::new("Median UMIs per cell", "867"),
        },
        valid_bc_read_frac: HeroMetric::new("Valid barcodes", "93.6%"),
    };
    assert_eq!(EXPECTED_TEMPLATE, content.template(None));
}

#[test]
fn test_html_template_vec() {
    const EXPECTED_TEMPLATE: &str = r#"<div class="row">
<div class="col">
<div data-key="hero_metrics[0]" data-component="Metric"></div>
</div>
</div>
<div class="row">
<div class="col">
<div data-key="hero_metrics[1]" data-component="Metric"></div>
</div>
</div>"#;
    let content = vec![
        HeroMetric::new("Number of cells", "3,487"),
        HeroMetric::new("Median UMIs per cell", "867"),
    ];
    assert_eq!(
        EXPECTED_TEMPLATE,
        content.template(Some("hero_metrics".into()))
    );
}

#[test]
fn test_html_template_grid() {
    const EXPECTED_TEMPLATE: &str = r#"<div class="row">
<div class="col-sm-6">
<div data-key="grid_data[0]" data-component="Metric"></div>
</div>
<div class="col-sm-6">
<div data-key="grid_data[1]" data-component="Metric"></div>
</div>
</div>
<div class="row">
<div class="col-sm-6">
<div data-key="grid_data[2]" data-component="Metric"></div>
</div>
</div>"#;
    let content = Grid::with_elements(
        vec![
            HeroMetric::new("Number of cells", "3,487"),
            HeroMetric::new("Median UMIs per cell", "867"),
            HeroMetric::new("Median Genes per cell", "700"),
        ],
        GridLayout::MaxCols(2),
    );
    assert_eq!(EXPECTED_TEMPLATE, content.template(None));
    assert_eq!(
        r#"{"grid_data":[{"metric":"3,487","name":"Number of cells","threshold":null},{"metric":"867","name":"Median UMIs per cell","threshold":null},{"metric":"700","name":"Median Genes per cell","threshold":null}]}"#,
        serde_json::to_string(&content).unwrap()
    );
}

#[test]
fn test_html_template_dyn_grid() {
    #[derive(Serialize, Clone, HtmlTemplate)]
    struct FullContent {
        grid: DynGrid,
    }
    const EXPECTED_TEMPLATE_1: &str = r#"<div class="row">
<div class="col-sm-6">
<div data-key="grid_data[0]" data-component="Metric"></div>
</div>
<div class="col-sm-6">
<div data-key="grid_data[1]" data-component="RawImage"></div>
</div>
</div>
<div class="row">
<div class="col-sm-6">
<div data-key="grid_data[2]" data-component="Metric"></div>
</div>
</div>"#;
    const EXPECTED_TEMPLATE_2: &str = r#"<div class="row">
<div class="col">
<div class="row">
<div class="col-sm-6">
<div data-key="grid.grid_data[0]" data-component="Metric"></div>
</div>
<div class="col-sm-6">
<div data-key="grid.grid_data[1]" data-component="RawImage"></div>
</div>
</div>
<div class="row">
<div class="col-sm-6">
<div data-key="grid.grid_data[2]" data-component="Metric"></div>
</div>
</div>
</div>
</div>
"#;
    let mut content = DynGrid::new(GridLayout::MaxCols(2));
    content.push(HeroMetric::new("Number of cells", "3,487"));
    content.push(RawImage::new("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAgAAAAIAQMAAAD+wSzIAAAABlBMVEX///+/v7+jQ3Y5AAAADklEQVQI12P4AIX8EAgALgAD/aNpbtEAAAAASUVORK5CYII".into()));
    content.push(HeroMetric::new("Median UMIs per cell", "867"));
    assert_eq!(EXPECTED_TEMPLATE_1, content.template(None));
    assert_eq!(
        EXPECTED_TEMPLATE_2,
        FullContent { grid: content }.template(None)
    );
}
