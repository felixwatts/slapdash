use std::num::FpCategory;

use sqlx::SqliteConnection;
use tracing::instrument;
use crate::db;
use axum::extract::{Path, State};
use crate::model::WidgetType;
use crate::view::{FreshnessWidgetTemplate, GaugeWidgetTemplate, LineWidgetTemplate, WidgetTemplateInner};
use crate::{model::{Dashboard, Widget}, view::{MainTemplate, WidgetTemplate}};
use axum::http::StatusCode;
use askama::Template;
use axum::response::Html;
use crate::env::Environment;

#[instrument(err, level = "error", skip_all)]
pub(crate) async fn get_default (
    State(env): State<Environment>,
) -> Result<Html<String>, StatusCode>
{
    _get("default", &env).await
}

#[instrument(err, level = "error", skip_all)]
pub(crate) async fn get (
    Path(dashboard): Path<String>, 
    State(env): State<Environment>,
) -> Result<Html<String>, StatusCode>
{
    _get(&dashboard, &env).await
}

#[instrument(err, level = "error", skip_all)]
async fn _get(dashboard_name: &str, env: &Environment) -> Result<Html<String>, StatusCode> {
    let mut db = env
        .db
        .acquire()
        .await
        .map_err(|e| {
            println!("Error while acquiring database connection: {}", e.to_string());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let dashboard = env.dashboards.get(dashboard_name)
        .ok_or(StatusCode::NOT_FOUND)?;

    let template = build_main(dashboard, &mut db)
        .await
        .map_err(|e| {
            println!("Error while building template: {}", e.to_string());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let html = template
        .render()
        .map_err(|e| {
            println!("Error while rendering template: {}", e.to_string());
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Html(html))
}

#[instrument(err, level = "error", skip_all)]
pub(crate) async fn put(
    Path((secret, series, value)): Path<(String, String, f32)>, 
    State(env): State<Environment>,
) -> Result<String, StatusCode> {
    if secret != env.settings.secret {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match value.classify() {
        FpCategory::Normal | FpCategory::Zero => {
            let mut db = env.db
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
            name: config.name.clone(),
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