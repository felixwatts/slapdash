use std::collections::HashMap;
use std::io::Read;
use std::net::SocketAddr;
use std::str::FromStr;
use std::env;
use serde::{Deserialize, Serialize};
use crate::model::{Color, Dashboard, Widget as ModelWidget, WidgetType};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::path::PathBuf;
use anyhow::anyhow;
use std::fs::{File, create_dir_all, write};
use sqlx::pool::PoolConnection;
use sqlx::Sqlite;

const DEFAULT_LISTEN_ADDR: &str = "127.0.0.1:8080";
const EMPTY_DASHBOARD: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<column xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="../dashboard.xsd">
    <label text="Hello, world!" width="12" />
</column>
"#;
const DASHBOARD_XSD: &str = include_str!("../dashboard.xsd");

#[derive(Clone)]
pub struct Environment{
    pub settings: Settings,
    pub dashboards: Dashboards,
    pub db: Db
}

impl Environment{
    pub async fn load() -> anyhow::Result<Self> {
        DashboardSchemaFile::init()?;
        Ok(
            Self{
                settings: Settings::load()?,
                dashboards: Dashboards::load()?,
                db: Db::init().await?
            }
        )
    }

    fn path() -> anyhow::Result<PathBuf> {
        let home_dir = env::var("HOME").map_err(|_| anyhow!(""))?;
        let config_dir = PathBuf::from(home_dir).join(".slapdash");
        Ok(config_dir)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub listen_addr: SocketAddr,
    pub secret: String
}

impl Settings {
    pub fn new() -> Self{
        Self { 
            listen_addr: SocketAddr::from_str(DEFAULT_LISTEN_ADDR).unwrap(), 
            secret: Self::generate_secret() 
        }
    }

    pub fn load() -> anyhow::Result<Self> {
        Self::init()?;

        let path = Self::path()?;
        let config_str = std::fs::read_to_string(&path)?;
        let settings: Settings = serde_ini::from_str(&config_str)?;
        Ok(settings)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path()?;
        let config_str = serde_ini::to_string(self)?;
        std::fs::write(&path, &config_str)?;
        Ok(())
    }

    fn init() -> anyhow::Result<()> {
        if !Self::path()?.exists() {
            let settings = Self::new();
            settings.save()?;
        }
        Ok(())
    }

    fn path() -> anyhow::Result<PathBuf> {
        let home_dir = env::var("HOME").map_err(|_| anyhow!(""))?;
        let config_dir = PathBuf::from(home_dir).join(".slapdash");
        let config_file = config_dir.join("config.txt");
        Ok(config_file)
    }

    fn generate_secret() -> String {
        let rng = thread_rng();
        // Generate a 64-character alphanumeric string
        let secret: String = rng
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();
        secret
    }
}

#[derive(Clone)]
pub struct Dashboards(HashMap<String, Dashboard>);

impl Dashboards{
    pub fn get(&self, name: &str) -> Option<&Dashboard> {
        self.0.get(name)
    }

    pub fn new_dashboard(name: &str) -> anyhow::Result<String> {
        let dashboard_file = Self::path()?.join(format!("{name}.xml"));
        if dashboard_file.exists() {
            return Ok(format!("Dashboard already exists: {}", dashboard_file.display()));
        }
        write(&dashboard_file, EMPTY_DASHBOARD)?;
        Ok(format!("Created a new dashboard at: {}", dashboard_file.display()))
    }

    pub fn list(&self) -> Vec<String> {
        self.0.keys().cloned().collect::<Vec<String>>()
    }

    fn load() -> anyhow::Result<Self> {
        Self::init()?;

        let mut dashboards = HashMap::new();
        for entry in std::fs::read_dir(Self::path()?)? {
            let entry = entry?;
            let file_name = entry.path().to_string_lossy().to_string();
            let dashboard = Self::load_dashboard(&file_name)?;
            let dashboard_name = entry.path().file_stem().unwrap().to_string_lossy().to_string();
            dashboards.insert(dashboard_name, dashboard);
        }
        Ok(Self(dashboards))
    }

    fn load_dashboard(file_name: &str) -> anyhow::Result<crate::model::Dashboard> {
        let mut file = File::open(file_name).map_err(anyhow::Error::from)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(anyhow::Error::from)?;
        let config: Widget = quick_xml::de::from_str(&contents).map_err(anyhow::Error::from)?;
        let dashboard = config.to_dashboard();
        Ok(dashboard)
    }

    fn init() -> anyhow::Result<()> {
        create_dir_all(Self::path()?)?;
        Self::new_dashboard("default")?;
        Ok(())
    }

    fn path() -> anyhow::Result<PathBuf> {
        Ok(Environment::path()?.join("dashboards"))
    }
}

struct DashboardSchemaFile{
}

impl DashboardSchemaFile{
    fn init() -> anyhow::Result<()> {
        if !Self::path()?.exists() {
            write(&Self::path()?, DASHBOARD_XSD)?;
        }
        Ok(())
    }

    fn path() -> anyhow::Result<PathBuf> {
        Ok(Environment::path()?.join("dashboard.xsd"))
    }
}

#[derive(Clone)]
pub struct Db(sqlx::SqlitePool);

impl Db{
    pub async fn acquire(&self) -> anyhow::Result<PoolConnection<Sqlite>> {
        Ok(self.0.acquire().await.map_err(anyhow::Error::from)?)
    }
    
    fn url() -> anyhow::Result<String> {
        Ok(format!("sqlite://{}?mode=rwc", &Environment::path()?.join("slapdash.db").display()))
    }

    async fn init() -> anyhow::Result<Self> {
        let pool = sqlx::sqlite::SqlitePool::connect(&Self::url()?).await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(Self(pool))
    }
}

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
    Gauge(Gauge),
    Line(Line),
    Value(Value),
    Row(Row),
    Column(Column),
}

impl Widget{
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
                left,
                top,
                width: widget.width.unwrap_or(default_width.unwrap_or(1)),
                height: widget.height.unwrap_or(default_height.unwrap_or(1)),
                series: widget.series.clone(),
                typ: WidgetType::Freshness,
                color: widget.color.clone().or(default_color.clone()),
            }),
            Widget::Gauge(widget) => Some(ModelWidget{
                label: widget.label.clone(),
                left,
                top,
                width: widget.width.unwrap_or(default_width.unwrap_or(1)),
                height: widget.height.unwrap_or(default_height.unwrap_or(1)),
                series: widget.series.clone(),
                typ: WidgetType::Gauge {
                    min: widget.min as f32,
                    max: widget.max as f32,
                },
                color: widget.color.clone().or(default_color.clone()),
            }),
            Widget::Line(widget) => Some(ModelWidget{
                label: widget.label.clone(),
                left,
                top,
                width: widget.width.unwrap_or(default_width.unwrap_or(1)),
                height: widget.height.unwrap_or(default_height.unwrap_or(1)),
                series: widget.series.clone(),
                typ: WidgetType::Line,
                color: widget.color.clone().or(default_color.clone()),
            }),
            Widget::Value(widget) => Some(ModelWidget{
                label: widget.label.clone(),
                left,
                top,
                width: widget.width.unwrap_or(default_width.unwrap_or(1)),
                height: widget.height.unwrap_or(default_height.unwrap_or(1)),
                series: widget.series.clone(),
                typ: WidgetType::Value,
                color: widget.color.clone().or(default_color.clone()),
            }),
            Widget::Label(widget) => Some(ModelWidget{
                label: widget.text.clone(),
                left,
                top,
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
            x => panic!("Invalid widget type: {x:?}")
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

/// Gauge widget with label, series, min, and max attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gauge {
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
        let _ = quick_xml::de::from_str::<Widget>(&xml_content).unwrap();
        
        // let dashboard = config.to_dashboard();
        // assert_eq!(dashboard.widgets.len(), 5);
    }

    #[test]
    fn test_complex_nested_conversion() {
        // Create a complex nested structure via XML:
        // Row 1: [Column 1: [Label, Value], Column 2: [Line]]
        // Row 2: [Column 3: [Gauge, Freshness]]
        
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
                    <gauge label="Gauge 1" series="series3" min="0" max="100" />
                    <freshness series="series4" />
                </column>
            </row>
        </column>
        "#;
        
        // Parse the XML into a Widget
        let config = quick_xml::de::from_str::<Widget>(xml_content).unwrap();
        
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