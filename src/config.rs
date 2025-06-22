use serde::{Deserialize, Serialize};
use crate::model::{Color, Dashboard, Widget as ModelWidget, WidgetType};

/// Root configuration element for slapdash
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub widgets: Vec<Widget>,
}

/// Row element with height and color attributes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Row {
    pub widgets: Vec<Widget>,
    pub widget_height: Option<u16>,
    pub widget_width: Option<u16>,
    pub widget_color: Option<Color>,
}

/// Column element with width attribute and various widget choices
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Column {
    pub widgets: Vec<Widget>,
    pub widget_height: Option<u16>,
    pub widget_width: Option<u16>,
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
    fn width(&self, default_widget_width: Option<u16>) -> u16 {
        match self {
            Widget::Column(column) => {
                column.widgets.iter().map(|widget| widget.width(column.widget_width.or(default_widget_width))).max().unwrap_or_default()
            },
            Widget::Row(row) => {
                row.widgets.iter().map(|widget| widget.width(row.widget_width.or(default_widget_width))).sum()
            },
            Widget::Freshness(widget) => widget.width.unwrap_or(default_widget_width.unwrap_or(1)),
            Widget::Value(widget) => widget.width.unwrap_or(default_widget_width.unwrap_or(1)),
            Widget::Line(widget) => widget.width.unwrap_or(default_widget_width.unwrap_or(1)),
            Widget::Gague(widget) => widget.width.unwrap_or(default_widget_width.unwrap_or(1)),
            Widget::Label(widget) => widget.width.unwrap_or(default_widget_width.unwrap_or(1)),
        }
    }

    fn height(&self, default_widget_height: Option<u16>) -> u16{
        match self {
            Widget::Column(column) => {
                column.widgets.iter().map(|widget| widget.height(column.widget_height.or(default_widget_height))).max().unwrap_or_default()
            },
            Widget::Row(row) => {
                row.widgets.iter().map(|widget| widget.height(row.widget_height.or(default_widget_height))).sum()
            },
            Widget::Freshness(widget) => widget.height.unwrap_or(default_widget_height.unwrap_or(1)),
            Widget::Value(widget) => widget.height.unwrap_or(default_widget_height.unwrap_or(1)),
            Widget::Line(widget) => widget.height.unwrap_or(default_widget_height.unwrap_or(1)),
            Widget::Gague(widget) => widget.height.unwrap_or(default_widget_height.unwrap_or(1)),
            Widget::Label(widget) => widget.height.unwrap_or(default_widget_height.unwrap_or(1)),
        }
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
                row.widgets.iter().fold((left, 0), |(left, container_height), widget| {
                    let  (width, height) = widget.to_model(
                        left, 
                        top, 
                        default_width, 
                        row.widget_height.or(default_height), 
                        row.widget_color.clone().or(default_color.clone()), 
                        models
                    );
                    (left + width, u16::max(container_height, height))
                })
            },
            Widget::Column(column) => {
                column.widgets.iter().fold((0, top), |(contaier_width, top), widget| {
                    let  (width, height) = widget.to_model(
                        left, 
                        top, 
                        column.widget_width.or(default_width), 
                        default_height, 
                        column.widget_color.clone().or(default_color.clone()), 
                        models
                    );
                    (u16::max(contaier_width, width), top + height)
                })
            },
            x => panic!("Invalid widget type: {:?}", x)
        }
    }
}

/// Label widget with text attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub text: String,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub color: Option<Color>,
}

/// Freshness widget with series attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Freshness {
    pub series: String,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub color: Option<Color>,
}

/// Gague widget with label, series, min, and max attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gague {
    pub label: String,
    pub series: String,
    pub min: f64,
    pub max: f64,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub color: Option<Color>,
}

/// Line widget with label and series attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub label: String,
    pub series: String,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub color: Option<Color>,
}

/// Value widget with label and series attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub label: String,
    pub series: String,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub color: Option<Color>,
}

impl Config {
    #[cfg(test)]
    pub fn new() -> Self {
        Config {
            widgets: Vec::new(),
        }
    }

    /// Convert this Config to a Dashboard
    pub fn to_dashboard(&self) -> Dashboard {
        let root = Widget::Column(Column{
            widgets: self.widgets.clone(),
            ..Default::default()
        });
        let mut widgets = Vec::new();
        root.to_model(1, 1, None, None, None, &mut widgets);
        
        Dashboard { widgets }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_slapdash_yaml() {
        // Read the JSON file
        let json_content = fs::read_to_string("slapdash.json")
            .expect("Failed to read slapdash.json file");

        // Parse the JSON into our Config struct
        let config: Config = serde_json::from_str(&json_content).unwrap();

        // Basic validation - check that we have widgets
        assert!(!config.widgets.is_empty(), "Config should have widgets");

        // // Count the total number of rows
        // let row_count = config.widgets.iter()
        //     .filter_map(|item| match item {
        //         Widget::Row(_) => Some(()),
        //         _ => None,
        //     })
        //     .count();
        
        // assert!(row_count > 0, "Should have at least one row");

        // // Check that we have the expected sections by looking for specific labels
        // let mut found_battery = false;
        // let mut found_water = false;
        // let mut found_solar = false;
        // let mut found_inverter = false;
        // let mut found_weather = false;
        // let mut found_cabin = false;

        // for element in &config.widgets {
        //     match element {
        //         Widget::Row(row) => {
        //             for column in &row.widgets {
        //                 for widget in &column.widgets {
        //                     if let Widget::Label(label) = widget {
        //                         if label.text.contains("🔋 Battery") {
        //                             found_battery = true;
        //                         } else if label.text.contains("🚰 Water") {
        //                             found_water = true;
        //                         } else if label.text.contains("🌞 Solar") {
        //                             found_solar = true;
        //                         } else if label.text.contains("∿ Inverter") {
        //                             found_inverter = true;
        //                         } else if label.text.contains("⛅ Weather") {
        //                             found_weather = true;
        //                         } else if label.text.contains("🏠 Cabin") {
        //                             found_cabin = true;
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //         _ => {}
        //     }
        // }

        // // Verify we found all the expected sections
        // assert!(found_battery, "Should have found Battery section");
        // assert!(found_water, "Should have found Water section");
        // assert!(found_solar, "Should have found Solar section");
        // assert!(found_inverter, "Should have found Inverter section");
        // assert!(found_weather, "Should have found Weather section");
        // assert!(found_cabin, "Should have found Cabin section");

        // // Test that we can serialize back to YAML
        // let serialized = serde_yml::to_string(&config)
        //     .expect("Failed to serialize Config back to YAML");
        
        // assert!(!serialized.is_empty(), "Serialized YAML should not be empty");
        // assert!(serialized.contains("slapdash:"), "Serialized YAML should contain root element");

        // println!("Successfully parsed slapdash.yaml with {} widgets", config.widgets.len());
        // println!("Found sections: Battery={}, Water={}, Solar={}, Inverter={}, Weather={}, Cabin={}", 
        //         found_battery, found_water, found_solar, found_inverter, found_weather, found_cabin);
    }

    #[test]
    fn test_config_creation() {
        let mut config = Config::new();
        
        // Create a simple test row
        let test_row = Row {
            widgets: vec![
                Widget::Column(
                    Column {
                        widget_color: Some(Color::Blue),
                        widgets: vec![
                            Widget::Label(Label {
                                text: "Test Label".to_string(),
                                width: None,
                                height: None,
                                color: None 
                            }),
                            Widget::Value(Value {
                                label: "Test Value".to_string(),
                                series: "test_series".to_string(),
                                width: None,
                                height: None,
                                color: None 
                            }),
                        ],
                        widget_width: Some(6),
                        ..Default::default()
                    }
                )
            ],
            widget_height: Some(3),
            widget_color: Some(Color::Blue),
            ..Default::default()
        };

        let widget = Widget::Row(test_row);

        assert_eq!(6, widget.width(None));
        assert_eq!(3, widget.height(None));

        config.widgets.push(widget);
        
        assert_eq!(config.widgets.len(), 1);
        
        if let Widget::Row(row) = &config.widgets[0] {
            assert_eq!(row.widget_height, Some(3));
            assert!(matches!(row.widget_color, Some(Color::Blue)));
            assert_eq!(row.widgets.len(), 1);
            // assert_eq!(row.widgets[0].widgets.len(), 2);
        } else {
            panic!("Expected a row element");
        }
    }

    #[test]
    fn test_config_to_dashboard_conversion() {
        let mut config = Config::new();
        
        // Create a simple test configuration with a row containing a column
        let test_column = Column {
            widgets: vec![
                Widget::Row(Row {
                    widget_height: Some(3),
                    widgets: vec![
                        Widget::Label(Label {
                            text: "Test Label".to_string(),
                            width: None,
                            height: None,
                            color: None 
                        })
                    ],
                    widget_color: None,
                    ..Default::default()
                }),

                Widget::Row(Row {
                    widget_height: Some(4),
                    widgets: vec![
                        Widget::Value(Value {
                            label: "Test Value".to_string(),
                            series: "test_series".to_string(),
                            width: None,
                            height: None,
                            color: None 
                        }),
                    ],
                    widget_color: None,
                    ..Default::default()
                }),
            ],
            widget_width: Some(6),
            widget_color: Some(Color::Pink),
            ..Default::default()
        };
        
        let test_row = Row {
            widgets: vec![Widget::Column(test_column)],
            widget_height: Some(3),
            widget_color: Some(Color::Red),
            ..Default::default()
        };

        let widget = Widget::Row(test_row);

        assert_eq!(6, widget.width(None));
        assert_eq!(4, widget.height(None));

        config.widgets.push(widget);
        
        // Convert to dashboard
        let dashboard = config.to_dashboard();
        
        // Verify we have the expected number of widgets (2: Label + Value)
        assert_eq!(dashboard.widgets.len(), 2);
        
        // Verify the label widget
        let label_widget = &dashboard.widgets[0];
        assert_eq!(label_widget.label, "Test Label");
        assert_eq!(label_widget.left, 1);
        assert_eq!(label_widget.top, 1);
        assert_eq!(label_widget.width, 6);
        assert_eq!(label_widget.height, 3);
        assert_eq!(label_widget.series, "");
        assert!(matches!(label_widget.typ, WidgetType::Label));
        
        // Verify the value widget
        let value_widget = &dashboard.widgets[1];
        assert_eq!(value_widget.label, "Test Value");
        assert_eq!(value_widget.left, 1);
        assert_eq!(value_widget.top, 4); // Should be below the label
        assert_eq!(value_widget.width, 6);
        assert_eq!(value_widget.height, 4);
        assert_eq!(value_widget.series, "test_series");
        assert!(matches!(value_widget.typ, WidgetType::Value));
    }

    #[test]
    fn test_complex_nested_conversion() {
        let mut config = Config::new();
        
        // Create a complex nested structure:
        // Row 1: [Column 1: [Label, Value], Column 2: [Line]]
        // Row 2: [Column 3: [Gague, Freshness]]
        
        let column1 = Column {
            widgets: vec![
                Widget::Label(Label {
                    text: "Section 1".to_string(),
                    width: None,
                    height: None,
                    color: None                     
                }),
                Widget::Value(Value {
                    label: "Value 1".to_string(),
                    series: "series1".to_string(),
                    width: None,
                    height: None,
                    color: None                     
                }),
            ],
            widget_width: Some(3),
            widget_color: Some(Color::Blue),
            ..Default::default()
        };
        
        let column2 = Column {
            widgets: vec![
                Widget::Line(Line {
                    label: "Chart 1".to_string(),
                    series: "series2".to_string(),
                    width: None,
                    height: None,
                    color: None                     
                }),
            ],
            widget_width: Some(3),
            widget_color: Some(Color::Green),
            ..Default::default()
        };
        
        let row1 = Row {
            widgets: vec![
                Widget::Column(column1),
                Widget::Column(column2),
            ],
            widget_height: Some(2),
            widget_color: Some(Color::Red),
            ..Default::default()
        };
        
        let column3 = Column {
            widgets: vec![
                Widget::Gague(Gague {
                    label: "Gauge 1".to_string(),
                    series: "series3".to_string(),
                    min: 0.0,
                    max: 100.0,
                    width: None,
                    height: None,
                    color: None                     
                }),
                Widget::Freshness(Freshness {
                    series: "series4".to_string(),
                    width: None,
                    height: None,
                    color: None                     
                }),
            ],
            widget_width: Some(6),
            widget_color: Some(Color::Yellow),
            ..Default::default()
        };
        
        let row2 = Row {
            widgets: vec![Widget::Column(column3)],
            widget_height: Some(2),
            widget_color: Some(Color::Purple),
            ..Default::default()
        };
        
        config.widgets.push(Widget::Row(row1));
        config.widgets.push(Widget::Row(row2));
        
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
                    assert_eq!(widget.top, 4); // Should be in row2
                    found_gauge1 = true;
                }
                "" => {
                    // Freshness widget has empty label
                    if widget.series == "series4" {
                        assert_eq!(widget.left, 0);
                        assert_eq!(widget.top, 3); // Should be below gauge in row2
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
