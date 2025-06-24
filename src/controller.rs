use std::num::FpCategory;

use sqlx::{AnyConnection, Sqlite, SqliteConnection};
use crate::config::Config;
use crate::db;
use axum::extract::{Path, State};
use crate::model::WidgetType;
use crate::view::{FreshnessWidgetTemplate, GaugeWidgetTemplate, LineWidgetTemplate, WidgetTemplateInner};
use crate::{model::{Dashboard, Widget}, view::{MainTemplate, WidgetTemplate}};
use sqlx::Acquire;
use axum::http::StatusCode;
use askama::Template;
use axum::response::Html;
use crate::AppState;

pub(crate) async fn get (
    Path(dashboard): Path<Option<String>>, 
    State(AppState { config, db }): State<AppState>,
) -> Result<Html<String>, StatusCode>
{
    let mut db = db.acquire().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dashboard_name = dashboard.unwrap_or("default".to_string());
    let dashboard = config.dashboards.get(&dashboard_name)
        .ok_or_else(|| StatusCode::NOT_FOUND)?;

    let template = build_main(dashboard, &mut db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let html = template
        .render()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Html(html))
}

pub(crate) async fn put(
    Path((secret, series, value)): Path<(String, String, f32)>, 
    State(AppState { config, db }): State<AppState>,
) -> Result<String, StatusCode> {
    if secret != config.settings.secret {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match value.classify() {
        FpCategory::Normal | FpCategory::Zero => {
            let mut db = db
                .acquire()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            db::put(&mut db, &series, value)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        },
        _ => {}
    }

    Ok("OK".to_string())
}

pub(crate) async fn build_main(config: &Dashboard, db: &mut SqliteConnection) -> anyhow::Result<MainTemplate> {
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

async fn build_widget(config: Widget, db: &mut SqliteConnection) -> anyhow::Result<WidgetTemplate>{
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