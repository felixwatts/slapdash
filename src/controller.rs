use std::num::FpCategory;

use sqlx::Sqlite;
use tide::{Response, StatusCode};
use tide_sqlx::SQLxRequestExt;
use crate::config::Config;
use crate::db;

use crate::model::WidgetType;
use crate::view::{FreshnessWidgetTemplate, GaugeWidgetTemplate, LineWidgetTemplate, WidgetTemplateInner};
use crate::{model::{Dashboard, Widget}, view::{MainTemplate, WidgetTemplate}};
use sqlx::Acquire;

pub(crate) async fn get(req: tide::Request<Config>) -> tide::Result {
    let mut db = req.sqlx_conn::<Sqlite>().await;
    let db = db.acquire().await?;
    let config = req.state();

    let dashboard_name = req.param("dashboard").unwrap_or("default");
    let dashboard = config.dashboards.get(dashboard_name)
        .ok_or_else(|| tide::Error::from_str(StatusCode::NotFound, "Dashboard not found"))?;

    let template = build_main(dashboard, db).await?;

    Ok(askama_tide::into_response(&template))
}

pub(crate) async fn put(req: tide::Request<Config>) -> tide::Result {
    let config = req.state();
    let actual_secret = req.param("secret")?;

    if actual_secret != config.settings.secret {
        return Err(tide::Error::from_str(StatusCode::Unauthorized, "Unauthorized"));
    }
    let series = req.param("series")?;
    let value_str = req.param("value")?;
    let value: f32 = value_str.parse()?;

    match value.classify() {
        FpCategory::Normal | FpCategory::Zero => {
            let mut db = req.sqlx_conn::<Sqlite>().await;
            let db = db.acquire().await?;
            db::put(db, series, value).await.map_err(|msg| tide::Error::from_str(StatusCode::InternalServerError, msg))?;
        },
        _ => {}
    }

    Ok(Response::builder(StatusCode::Ok).build())
}

pub(crate) async fn build_main(config: &Dashboard, db: &mut sqlx::SqliteConnection) -> tide::Result<MainTemplate> {
    let mut widget_templates = vec![];
    for widget_config in config.widgets.iter() {
        let widget_template = build_widget(widget_config.clone(), db).await?;
        widget_templates.push(widget_template)
    }

    Ok(
        MainTemplate{
            width: config.width(),
            height: config.height(),
            widgets: widget_templates
        }
    )
}

async fn build_widget(config: Widget, db: &mut sqlx::SqliteConnection) -> tide::Result<WidgetTemplate>{
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
        ),
        WidgetType::Gauge{ .. } => WidgetTemplateInner::Gauge(
            GaugeWidgetTemplate{
                config: config.clone(),
                point: data.last().map(|p| p.value) 
            }
        ),
        WidgetType::Label => WidgetTemplateInner::Label,
        WidgetType::Freshness => WidgetTemplateInner::Freshness(
            FreshnessWidgetTemplate{
                last_update_time: data.last().map(|p| p.time) 
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