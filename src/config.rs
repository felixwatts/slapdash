use serde::{Deserialize, Serialize};
use quick_xml::de;
use crate::model::{Color, Dashboard, Widget as ModelWidget, WidgetType};

/// Row element with height and color attributes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Row {
    #[serde(rename = "$value")]
    pub widgets: Vec<Widget>,
    #[serde(rename = "@widget_height")]
    pub widget_height: Option<u16>,
    #[serde(rename = "@widget_width")]
    pub widget_width: Option<u16>,
    #[serde(rename = "@widget_color")]
    pub widget_color: Option<Color>,
}

/// Column element with width attribute and various widget choices
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Column {
    #[serde(rename = "$value")]
    pub widgets: Vec<Widget>,
    #[serde(rename = "@widget_height")]
    pub widget_height: Option<u16>,
    #[serde(rename = "@widget_width")]
    pub widget_width: Option<u16>,
    #[serde(rename = "@widget_color")]
    pub widget_color: Option<Color>,
}

/// Enum representing the various widget types that can appear in a column
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Widget {
    Label(Label),
    Freshness(Freshness),
    Gague(Gague),
    Line(Line),
    Value(Value),
    Row(Row),
    Column(Column),
}

impl Widget{
    // fn width(&self, default_widget_width: Option<u16>) -> u16 {
    //     match self {
    //         Widget::Column(column) => {
    //             column.widgets.iter().map(|widget| widget.width(column.widget_width.or(default_widget_width))).max().unwrap_or_default()
    //         },
    //         Widget::Row(row) => {
    //             row.widgets.iter().map(|widget| widget.width(row.widget_width.or(default_widget_width))).sum()
    //         },
    //         Widget::Freshness(widget) => widget.width.unwrap_or(default_widget_width.unwrap_or(1)),
    //         Widget::Value(widget) => widget.width.unwrap_or(default_widget_width.unwrap_or(1)),
    //         Widget::Line(widget) => widget.width.unwrap_or(default_widget_width.unwrap_or(1)),
    //         Widget::Gague(widget) => widget.width.unwrap_or(default_widget_width.unwrap_or(1)),
    //         Widget::Label(widget) => widget.width.unwrap_or(default_widget_width.unwrap_or(1)),
    //     }
    // }

    // fn height(&self, default_widget_height: Option<u16>) -> u16{
    //     match self {
    //         Widget::Column(column) => {
    //             column.widgets.iter().map(|widget| widget.height(column.widget_height.or(default_widget_height))).sum()
    //         },
    //         Widget::Row(row) => {
    //             row.widgets.iter().map(|widget| widget.height(row.widget_height.or(default_widget_height))).max().unwrap_or_default()
    //         },
    //         Widget::Freshness(widget) => widget.height.unwrap_or(default_widget_height.unwrap_or(1)),
    //         Widget::Value(widget) => widget.height.unwrap_or(default_widget_height.unwrap_or(1)),
    //         Widget::Line(widget) => widget.height.unwrap_or(default_widget_height.unwrap_or(1)),
    //         Widget::Gague(widget) => widget.height.unwrap_or(default_widget_height.unwrap_or(1)),
    //         Widget::Label(widget) => widget.height.unwrap_or(default_widget_height.unwrap_or(1)),
    //     }
    // }

    pub(crate) fn to_dashboard(&self) -> Dashboard {
        let mut widgets = Vec::new();
        self.to_model(1, 1, None, None, None, &mut widgets);
        Dashboard { widgets }
    }

    fn to_model(
        &self, 
        left: u16, 
        top: u16, 
        default_width: Option<u16>, 
        default_height: Option<u16>, 
        default_color: Option<Color>,
        models: &mut Vec<ModelWidget>
    ) -> (u16, u16) {
        let model= match self{
            Widget::Freshness(widget) => Some(ModelWidget{
                label: String::new(),
                left: left,
                top: top,
                width: widget.width.unwrap_or(default_width.unwrap_or(1)),
                height: widget.height.unwrap_or(default_height.unwrap_or(1)),
                series: widget.series.clone(),
                typ: WidgetType::Freshness,
                color: widget.color.clone().or(default_color.clone()),
            }),
            Widget::Gague(widget) => Some(ModelWidget{
                label: widget.label.clone(),
                left: left,
                top: top,
                width: widget.width.unwrap_or(default_width.unwrap_or(1)),
                height: widget.height.unwrap_or(default_height.unwrap_or(1)),
                series: widget.series.clone(),
                typ: WidgetType::Gague {
                    min: widget.min as f32,
                    max: widget.max as f32,
                },
                color: widget.color.clone().or(default_color.clone()),
            }),
            Widget::Line(widget) => Some(ModelWidget{
                label: widget.label.clone(),
                left: left,
                top: top,
                width: widget.width.unwrap_or(default_width.unwrap_or(1)),
                height: widget.height.unwrap_or(default_height.unwrap_or(1)),
                series: widget.series.clone(),
                typ: WidgetType::Line,
                color: widget.color.clone().or(default_color.clone()),
            }),
            Widget::Value(widget) => Some(ModelWidget{
                label: widget.label.clone(),
                left: left,
                top: top,
                width: widget.width.unwrap_or(default_width.unwrap_or(1)),
                height: widget.height.unwrap_or(default_height.unwrap_or(1)),
                series: widget.series.clone(),
                typ: WidgetType::Value,
                color: widget.color.clone().or(default_color.clone()),
            }),
            Widget::Label(widget) => Some(ModelWidget{
                label: widget.text.clone(),
                left: left,
                top: top,
                width: widget.width.unwrap_or(default_width.unwrap_or(1)),
                height: widget.height.unwrap_or(default_height.unwrap_or(1)),
                series: String::new(),
                typ: WidgetType::Label,
                color: widget.color.clone().or(default_color.clone()),
            }),
            _ => None
        };

        if let Some(model) = model {
            let return_value = (model.width, model.height);
            models.push(model);
            return return_value;
        }
        
        match self{
            Widget::Row(row) => {
                let mut current_height = 0;
                let mut current_width = 0;
                for widget in row.widgets.iter() {
                    let (widget_width, widget_height) = widget.to_model(
                        left + current_width, 
                        top, 
                        row.widget_width.or(default_width), 
                        row.widget_height.or(default_height), 
                        row.widget_color.clone().or(default_color.clone()),
                        models
                    );
                    current_width += widget_width;
                    current_height = u16::max(current_height, widget_height);
                }
                (current_width, current_height)
            },
            Widget::Column(column) => {
                let mut current_height = 0;
                let mut current_width = 0;
                for widget in column.widgets.iter() {
                    let (widget_width, widget_height) = widget.to_model(
                        left, 
                        top + current_height, 
                        column.widget_width.or(default_width), 
                        column.widget_height.or(default_height), 
                        column.widget_color.clone().or(default_color.clone()),
                        models
                    );
                    current_height += widget_height;
                    current_width = u16::max(current_width, widget_width);
                }
                (current_width, current_height)
            },
            x => panic!("Invalid widget type: {:?}", x)
        }
    }
}

/// Label widget with text attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    #[serde(rename = "@text")]
    pub text: String,
    #[serde(rename = "@width")]
    pub width: Option<u16>,
    #[serde(rename = "@height")]
    pub height: Option<u16>,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
}

/// Freshness widget with series attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Freshness {
    #[serde(rename = "@series")]
    pub series: String,
    #[serde(rename = "@width")]
    pub width: Option<u16>,
    #[serde(rename = "@height")]
    pub height: Option<u16>,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
}

/// Gague widget with label, series, min, and max attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gague {
    #[serde(rename = "@label")]
    pub label: String,
    #[serde(rename = "@series")]
    pub series: String,
    #[serde(rename = "@min")]
    pub min: f64,
    #[serde(rename = "@max")]
    pub max: f64,
    #[serde(rename = "@width")]
    pub width: Option<u16>,
    #[serde(rename = "@height")]
    pub height: Option<u16>,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
}

/// Line widget with label and series attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    #[serde(rename = "@label")]
    pub label: String,
    #[serde(rename = "@series")]
    pub series: String,
    #[serde(rename = "@width")]
    pub width: Option<u16>,
    #[serde(rename = "@height")]
    pub height: Option<u16>,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
}

/// Value widget with label and series attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    #[serde(rename = "@label")]
    pub label: String,
    #[serde(rename = "@series")]
    pub series: String,
    #[serde(rename = "@width")]
    pub width: Option<u16>,
    #[serde(rename = "@height")]
    pub height: Option<u16>,
    #[serde(rename = "@color")]
    pub color: Option<Color>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_deserialize_slapdash_xml() {
        let xml_content = fs::read_to_string("slapdash.xml").unwrap();
        
        // Use the new from_xml method
        let config = de::from_str::<Widget>(&xml_content).unwrap();
        
        // let dashboard = config.to_dashboard();
        // assert_eq!(dashboard.widgets.len(), 5);
    }

    #[test]
    fn test_complex_nested_conversion() {
        // Create a complex nested structure via XML:
        // Row 1: [Column 1: [Label, Value], Column 2: [Line]]
        // Row 2: [Column 3: [Gague, Freshness]]
        
        let xml_content = r#"
        <column>
            <row widget_height="2" widget_color="Red">
                <column widget_width="3" widget_color="Blue">
                    <label text="Section 1" />
                    <value label="Value 1" series="series1" />
                </column>
                <column widget_width="3" widget_color="Green">
                    <line label="Chart 1" series="series2" />
                </column>
            </row>
            <row widget_height="2" widget_color="Purple">
                <column widget_width="6" widget_color="Yellow">
                    <gague label="Gauge 1" series="series3" min="0" max="100" />
                    <freshness series="series4" />
                </column>
            </row>
        </column>
        "#;
        
        // Parse the XML into a Widget
        let config = de::from_str::<Widget>(xml_content).unwrap();
        
        // Convert to dashboard
        let dashboard = config.to_dashboard();
        
        // Should have 5 widgets total: 2 in row1 + 2 in row2
        assert_eq!(dashboard.widgets.len(), 5);
        
        // Verify positioning:
        // Row 1 widgets should be at top=0
        // Row 2 widgets should be at top=2 (after row1 height)
        
        let mut found_section1 = false;
        let mut found_value1 = false;
        let mut found_chart1 = false;
        let mut found_gauge1 = false;
        let mut found_freshness = false;
        
        for widget in &dashboard.widgets {
            match widget.label.as_str() {
                "Section 1" => {
                    assert_eq!(widget.left, 1);
                    assert_eq!(widget.top, 1);
                    found_section1 = true;
                }
                "Value 1" => {
                    assert_eq!(widget.left, 1);
                    assert_eq!(widget.top, 3);
                    found_value1 = true;
                }
                "Chart 1" => {
                    assert_eq!(widget.left, 4); // Should be to the right of column1
                    assert_eq!(widget.top, 1);
                    found_chart1 = true;
                }
                "Gauge 1" => {
                    assert_eq!(widget.left, 1);
                    assert_eq!(widget.top, 5); // Should be in row2
                    found_gauge1 = true;
                }
                "" => {
                    // Freshness widget has empty label
                    if widget.series == "series4" {
                        assert_eq!(widget.left, 1);
                        assert_eq!(widget.top, 7); // Should be below gauge in row2
                        found_freshness = true;
                    }
                }
                _ => {}
            }
        }
        
        assert!(found_section1, "Should find Section 1 widget");
        assert!(found_value1, "Should find Value 1 widget");
        assert!(found_chart1, "Should find Chart 1 widget");
        assert!(found_gauge1, "Should find Gauge 1 widget");
        assert!(found_freshness, "Should find Freshness widget");
    }
}
