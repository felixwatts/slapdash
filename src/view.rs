use askama::Template;
use crate::model::*;

#[derive(Template)]
#[template(path = "main.html")]
pub (crate) struct MainTemplate {
    pub config: MainConfig,
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
    Line(LineWidgetTemplate)
}

#[derive(Template)]
#[template(path = "widget_value.html")]
pub (crate) struct ValueWidgetTemplate{
    pub config: WidgetConfig,
    pub point: Option<f32>
}

#[derive(Template)]
#[template(path = "widget_line.html")]
pub (crate) struct LineWidgetTemplate{
    pub config: WidgetConfig,
    pub data: Vec<Point>
}

impl LineWidgetTemplate{
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

        let x_range = (x_max - x_min) as f32;
        let normalize_x = |x: i64| {
            (((x - x_min) as f32) / x_range) * 100.0
        };

        let normalize_y = |y: f32| {
            100.0 - (((y - y_min) / (y_max - y_min)) * 100.0)
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
}
