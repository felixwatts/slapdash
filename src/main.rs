#[macro_use]
extern crate dotenv_codegen;

use std::fs::File;
use std::io::Read;

use model::Config;
use sqlx::prelude::*;
use sqlx::postgres::Postgres;
use tide::Response;
use tide::StatusCode;
use tide_sqlx::SQLxMiddleware;
use tide_sqlx::SQLxRequestExt;
use askama::Template;
use view::MainTemplate;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let config = load_config()?;

    let mut app = tide::with_state(config);

    app.with(SQLxMiddleware::<Postgres>::new(dotenv!("DATABASE_URL")).await?);

    app.at("/").get(get);
    app.at("/:secret/:series/:value").put(put);

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

fn load_config() -> tide::Result<model::Config> {
    let mut file = File::open("slapdash.json").map_err(|e| tide::Error::from_display(e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(|e| tide::Error::from_display(e))?;
    Ok(serde_json::from_str(&contents).map_err(|e| tide::Error::from_display(e))?)
}

async fn get(req: tide::Request<Config>) -> tide::Result {
    let mut db = req.sqlx_conn::<Postgres>().await;
    let config = req.state();

    let template = controller::build_main(config, db.acquire().await?).await?;

    Ok(askama_tide::into_response(&template))
}

async fn put(req: tide::Request<Config>) -> tide::Result {
    let actual_secret = req.param("secret")?;
    if actual_secret != dotenv!("SECRET") {
        return Err(tide::Error::from_str(StatusCode::Unauthorized, "Unauthorized"));
    }
    let series = req.param("series")?;
    let value_str = req.param("value")?;
    let value: f32 = value_str.parse()?;
    let mut db = req.sqlx_conn::<Postgres>().await;
    db::put(db.acquire().await?, series, value).await.map_err(|msg| tide::Error::from_str(StatusCode::InternalServerError, msg))?;
    Ok(Response::builder(StatusCode::Ok).build())
}

mod controller{
    use sqlx::PgConnection;
    use crate::db;

    use crate::model::WidgetType;
    use crate::view::{LineWidgetTemplate, WidgetTemplateInner};
    use crate::{model::{Config, WidgetConfig}, view::{MainTemplate, WidgetTemplate}};

    pub(crate) async fn build_main(config: &Config, db: &mut PgConnection) -> tide::Result<MainTemplate> {
        let mut widget_templates = vec![];
        for (id, widget_config) in config.widgets.iter().enumerate() {
            let widget_template = build_widget(id, widget_config.clone(), db).await?;
            widget_templates.push(widget_template)
        }

        Ok(
            MainTemplate{
                config: config.main.clone(),
                widgets: widget_templates
            }
        )
    }

    async fn build_widget(id: usize, config: WidgetConfig, db: &mut PgConnection) -> tide::Result<WidgetTemplate>{
        let data = db::get(db, &config.series).await?;
        let template = match config.typ {
            WidgetType::Value => WidgetTemplateInner::Value(
                crate::view::ValueWidgetTemplate { 
                    config: config.clone(), 
                    point: data.last().map(|p| p.value) 
                }
            ),
            WidgetType::Line => WidgetTemplateInner::Line(
                LineWidgetTemplate{
                    config: config.clone(),
                    data,
                }
            )
        };
        Ok(
            WidgetTemplate{
                config,
                template,
            }
        )
    }
}

mod view {
    use askama::{filters::format, Template};
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
}

mod model {
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
}

mod db {
    use sqlx::PgConnection;
    use crate::model::*;

    pub(crate) async fn put(db: &mut PgConnection, series: &str, point: f32) -> tide::Result<()> {
        sqlx::query!("
            WITH series_row AS (
                INSERT INTO series (name)
                VALUES ($1)
                ON CONFLICT (name) DO UPDATE
                SET name = EXCLUDED.name -- This is a no-op, just to handle conflict
                RETURNING id
            )
            INSERT INTO point (series_id, time, value)
            VALUES (
                (SELECT id FROM series_row),
                NOW(), -- Assuming you want to use the current time
                $2
            );
        ",
        series,
        point
        )
        .execute(db)
        .await
        .map_err(|e| tide::Error::from_display(e))?;

        Ok(())
    }

    pub(crate) async fn get(db: &mut PgConnection, series: &str) -> tide::Result<Vec<Point>>{
        let points = sqlx::query_as!(
            Point,
            "
            SELECT time, value
            FROM point
            INNER JOIN series ON point.series_id = series.id
            WHERE 
                series.name = $1
                AND time > NOW() - INTERVAL '24 hours'
            ORDER BY time DESC
            ",
            series
        )
        .fetch_all(db)
        .await
        .map_err(|e| tide::Error::from_display(e))?;

        Ok(points)
    }
}