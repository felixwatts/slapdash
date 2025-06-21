use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize, Deserialize, Clone, Default)]
pub(crate) enum Color{
    Red,
    Pink,
    Purple,
    DeepPurple,
    Indigo,
    Blue,
    LightBlue,
    Cyan,
    Aqua,
    #[default]
    Teal,
    Green,
    LightGreen,
    Lime,
    Sand,
    Khaki,
    Yellow,
    Amber,
    Orange,
    DeepOrange,
    BlueGray,
    Brown,
    LightGray,
    Gray,
    DarkGray,
    PaleRed,
    PaleYellow,
    PaleGreen,
    PaleBlue,
}

impl Color{
    pub fn to_css_class(&self) -> &'static str {
        match self{
            Color::Red => "w3-red",
            Color::Pink => "w3-pink",
            Color::Purple => "w3-purple",
            Color::DeepPurple => "w3-deep-purple",
            Color::Indigo => "w3-indigo",
            Color::Blue => "w3-blue",
            Color::LightBlue => "w3-light-blue",
            Color::Cyan => "w3-cyan",
            Color::Aqua => "w3-aqua",
            Color::Teal => "w3-teal",
            Color::Green => "w3-green",
            Color::LightGreen => "w3-light-green",
            Color::Lime => "w3-lime",
            Color::Sand => "w3-sand",
            Color::Khaki => "w3-khaki",
            Color::Yellow => "w3-yellow",
            Color::Amber => "w3-amber",
            Color::Orange => "w3-orange",
            Color::DeepOrange => "w3-deep-orange",
            Color::BlueGray => "w3-blue-gray",
            Color::Brown => "w3-brown",
            Color::LightGray => "w3-light-gray",
            Color::Gray => "w3-gray",
            Color::DarkGray => "w3-dark-gray",
            Color::PaleRed => "w3-pale-red",
            Color::PaleYellow => "w3-pale-yellow",
            Color::PaleGreen => "w3-pale-green",
            Color::PaleBlue => "w3-pale-blue",
        }
    }

    pub fn to_css_stroke(&self) -> &'static str {
        match self{
            Color::Red => "white",
            Color::Pink => "white",
            Color::Purple => "white",
            Color::DeepPurple => "white",
            Color::Indigo => "white",
            Color::Blue => "white",
            Color::LightBlue => "black",
            Color::Cyan => "black",
            Color::Aqua => "black",
            Color::Teal => "white",
            Color::Green => "white",
            Color::LightGreen => "black",
            Color::Lime => "black",
            Color::Sand => "black",
            Color::Khaki => "black",
            Color::Yellow => "black",
            Color::Amber => "black",
            Color::Orange => "black",
            Color::DeepOrange => "white",
            Color::BlueGray => "white",
            Color::Brown => "white",
            Color::LightGray => "black",
            Color::Gray => "black",
            Color::DarkGray => "white",
            Color::PaleRed => "black",
            Color::PaleYellow => "black",
            Color::PaleGreen => "black",
            Color::PaleBlue => "black",
        }
    }
}

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
pub(crate) struct Dashboard{
    pub widgets: Vec<Widget>
}

impl Dashboard {
    pub fn width(&self) -> u16 {
        self
            .widgets
            .iter()
            .map(|w| w.left + w.width - 1)
            .max()
            .unwrap_or_default()
    }

    pub fn height(&self) -> u16 {
        self
            .widgets
            .iter()
            .map(|w| w.top + w.height - 1)
            .max()
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Widget{
    pub label: String,
    pub left: u16,
    pub top: u16,
    pub width: u16,
    pub height: u16,
    pub series: String,
    pub typ: WidgetType,
    pub color: Option<Color>
}

impl Widget{
    pub(crate) fn color_css_class(&self) -> &str {
        self.color.as_ref().map(Color::to_css_class).unwrap_or(Color::default().to_css_class())
    }

    pub(crate) fn stroke_css_color(&self) -> &str {
        self.color.as_ref().map(Color::to_css_stroke).unwrap_or(Color::default().to_css_stroke())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) enum WidgetType{
    Value,
    Line,
    Gague{ min: f32, max:f32 },
    Label,
    Freshness
}