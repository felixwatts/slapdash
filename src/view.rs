use std::f32::consts::PI;
use std::fmt::Write;
use askama::Template;
use chrono::NaiveDateTime;
use crate::model::*;

#[derive(Template)]
#[template(path = "main.html")]
pub (crate) struct MainTemplate {
    pub width: u16,
    pub height: u16,
    pub widgets: Vec<WidgetTemplate>
}

#[derive(Template)]
#[template(path = "widget.html")]
pub (crate) struct WidgetTemplate{
    pub config: Widget,
    pub template: WidgetTemplateInner
}

pub (crate) enum WidgetTemplateInner{
    Value(ValueWidgetTemplate),
    Line(LineWidgetTemplate),
    Gauge(GaugeWidgetTemplate),
    Label,
    Freshness(FreshnessWidgetTemplate)
}

#[derive(Template)]
#[template(path = "widget_value.html")]
pub (crate) struct ValueWidgetTemplate{
    pub config: Widget,
    pub point: Option<f32>
}

impl ValueWidgetTemplate{
    pub fn text(&self) -> String {
        match self.point {
            Some(value) => format!("{value:.2}"),
            None => "N/A".into()
        }
    }
}   

#[derive(Template)]
#[template(path = "widget_freshness.html")]
pub (crate) struct FreshnessWidgetTemplate{
    pub last_update_time: Option<chrono::NaiveDateTime>
}

impl FreshnessWidgetTemplate{
    pub(crate) fn freshness(&self) -> String{
        match self.last_update_time {
            Some(time) => {
                let age = chrono::Utc::now().naive_utc() - time;
                format!("{} mins", &age.num_minutes())
            },
            None => "N/A".into()
        }
    }
}

#[derive(Template)]
#[template(path = "widget_gauge.html")]
pub (crate) struct GaugeWidgetTemplate{
    pub config: Widget,
    pub point: Option<f32>
}

impl GaugeWidgetTemplate{
    pub fn arc_svg(&self) -> String {
        match self.config.typ {
            WidgetType::Gauge { min, max } => {
                match self.point {
                    Some(mut value) => {
                        value = value.clamp(min, max);
                        let proportion = (value - min) / (max - min);
                        let angle = proportion * 2.0 * PI;
                        let radius = 38.0;
                        let start_x = 50.0 + radius * (-PI/2.0).cos();
                        let start_y = 50.0 + radius * (-PI/2.0).sin();
                        
                        // For full circle, use a point slightly before the end to avoid start/end point collision
                        let end_angle = if proportion >= 1.0 {
                            angle - 0.0001
                        } else {
                            angle
                        };
                        let end_x = 50.0 + radius * (end_angle - (PI/2.0)).cos();
                        let end_y = 50.0 + radius * (end_angle - (PI/2.0)).sin();

                        let large_arc_flag = if angle > PI { "1" } else { "0" };

                        format!("M {start_x} {start_y} A {radius} {radius} 1 {large_arc_flag} 1 {end_x} {end_y}")
                    },
                    None => String::default()
                }
            },
            _ => panic!()
        }
    }
}

#[derive(Template)]
#[template(path = "widget_line.html")]
pub (crate) struct LineWidgetTemplate{
    pub config: Widget,
    pub data: Vec<Point>
}

impl LineWidgetTemplate{
    pub fn axis_label_bottom(&self) -> String {
        self
            .data
            .iter()
            .map(|point| point.value)
            .min_by(f32::total_cmp)
            .map(|v| format!("{v:.1}"))
            .unwrap_or_default()
    }

    pub fn axis_label_top(&self) -> String {
        self
            .data
            .iter()
            .map(|point| point.value)
            .max_by(f32::total_cmp)
            .map(|v| format!("{v:.1}"))
            .unwrap_or_default()
    }

    pub fn points_svg(&self) -> String {
        if self.data.is_empty() {
            return String::default();
        }

        let xs = self.data.iter().map(|point| point.x()).collect::<Vec<_>>();
        let x_min = xs.iter().cloned().min().unwrap();
        let x_max = xs.iter().cloned().max().unwrap();

        if x_max == x_min {
            return String::default();
        }

        let y_min = self.data.iter().map(|point| point.value).min_by(f32::total_cmp).unwrap();
        let y_max = self.data.iter().map(|point| point.value).max_by(f32::total_cmp).unwrap();

        if y_max == y_min {
            return String::default();
        }

        let view_box_width = self.view_box_width();
        let view_box_height = self.view_box_height();

        let x_range = (x_max - x_min) as f32;
        let normalize_x = |x: i64| {
            (((x - x_min) as f32) / x_range) * view_box_width
        };

        let normalize_y = |y: f32| {
            view_box_height - (((y - y_min) / (y_max - y_min)) * view_box_height)
        };

        let result: String = self
            .data
            .iter()
            .map(|point|
                (
                    normalize_x(point.x()),
                    normalize_y(point.value)
                )
            )
            .fold(String::new(), |mut s, (x, y)| { write!(s, "{x},{y} ").unwrap(); s});

        result
    }

    pub fn view_box_width(&self) -> f32 {
        self.config.width as f32 * 100.0
    }

    pub fn view_box_height(&self) -> f32 {
        (self.config.height - 1) as f32 * 100.0
    }

    pub fn y_axis_left(&self) -> f32 {
        self.view_box_width() - 4.0
    }

    pub fn y_axis_bottom(&self) -> f32 {
        self.view_box_height()
    }
}
