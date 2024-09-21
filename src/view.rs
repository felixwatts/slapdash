use std::{f32::consts::PI, fmt::Debug};

use askama::Template;
use time::PrimitiveDateTime;
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
    pub config: WidgetConfig,
    pub template: WidgetTemplateInner
}

pub (crate) enum WidgetTemplateInner{
    Value(ValueWidgetTemplate),
    Line(LineWidgetTemplate),
    Gague(GagueWidgetTemplate),
    Label,
    Freshness(FreshnessWidgetTemplate)
}

#[derive(Template)]
#[template(path = "widget_value.html")]
pub (crate) struct ValueWidgetTemplate{
    pub point: Option<f32>
}

#[derive(Template)]
#[template(path = "widget_freshness.html")]
pub (crate) struct FreshnessWidgetTemplate{
    pub last_update_time: Option<time::PrimitiveDateTime>
}

impl FreshnessWidgetTemplate{
    pub(crate) fn freshness(&self) -> String{
        match self.last_update_time {
            Some(time) => {
                let age = PrimitiveDateTime::now() - time;
                format!("{}:{}:{}", &age.whole_hours(), &age.whole_minutes(), &age.whole_seconds())
            },
            None => "No data".into()
        }
    }
}

#[derive(Template)]
#[template(path = "widget_gague.html")]
pub (crate) struct GagueWidgetTemplate{
    pub config: WidgetConfig,
    pub point: Option<f32>
}

impl GagueWidgetTemplate{
    pub fn arc_svg(&self) -> String {
        match self.config.typ {
            WidgetType::Gague { min, max } => {
                match self.point {
                    Some(value) => {
                        let proportion = (value - min) / (max - min);
                        let angle = proportion * 2.0 * PI;
                        let end_x = 50.0 + 38.0 * (angle - (PI/2.0)).cos();
                        let end_y = 50.0 + 38.0 * (angle - (PI/2.0)).sin();

                        let large_arc_flag = if angle > PI { "1" } else { "0" };

                        format!("M 50 12 A 38 38 1 {large_arc_flag} 1 {end_x} {end_y}")
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
    pub config: WidgetConfig,
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
        if self.data.len() == 0 {
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
            .map(|(x, y)| format!("{x},{y} "))
            .collect();

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
