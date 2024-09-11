use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub(crate) struct Point{
    pub time: time::PrimitiveDateTime,
    pub value: f32
}

impl Point{
    pub fn x(&self) -> i64 {
        self.time.assume_utc().unix_timestamp()
    }
}

#[derive(Deserialize, Clone)]
pub(crate) struct Config{
    pub main: MainConfig,
    pub widgets: Vec<WidgetConfig>
}

#[derive(Deserialize, Clone)]
pub(crate) struct MainConfig{
    pub width: u16,
    pub height: u16
}

#[derive(Deserialize, Clone)]
pub(crate) struct WidgetConfig{
    pub left: u16,
    pub top: u16,
    pub width: u16,
    pub height: u16,
    pub series: String,
    pub typ: WidgetType
}

#[derive(Deserialize, Clone)]
pub(crate) enum WidgetType{
    Value,
    Line
}