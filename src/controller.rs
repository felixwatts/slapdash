use std::num::FpCategory;
use sqlx::SqliteConnection;
use crate::db;
use axum::extract::{Path, Query, State};
use crate::model::WidgetType;
use crate::view::{FreshnessWidgetTemplate, GaugeWidgetTemplate, LabelWidgetTemplate, LineWidgetTemplate, RangeWidgetTemplate, WidgetTemplateInner};
use crate::{model::{Dashboard, Widget}, view::{MainTemplate, WidgetTemplate}};
use axum::http::StatusCode;
use askama::Template;
use axum::response::Html;
use crate::env::Environment;
use serde::Deserialize;

const DEFAULT_RANGE_SECONDS: u32 = 86400;

#[derive(Deserialize)]
pub(crate) struct DashboardQuery {
    range: Option<u32>,
}

pub(crate) async fn get_default (
    Query(query): Query<DashboardQuery>,
    State(env): State<Environment>,
) -> Result<Html<String>, StatusCode>
{
    let range = resolve_range(query.range)?;
    _get("default", &env, range).await
}

pub(crate) async fn get (
    Path(dashboard): Path<String>,
    Query(query): Query<DashboardQuery>,
    State(env): State<Environment>,
) -> Result<Html<String>, StatusCode>
{
    let range = resolve_range(query.range)?;
    _get(&dashboard, &env, range).await
}

fn resolve_range(range: Option<u32>) -> Result<u32, StatusCode> {
    match range {
        None => Ok(DEFAULT_RANGE_SECONDS),
        Some(r) if r > 0 => Ok(r),
        Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn _get(dashboard_name: &str, env: &Environment, range_seconds: u32) -> Result<Html<String>, StatusCode> {
    let mut db = env
        .db
        .acquire()
        .await
        .map_err(|e| {
            println!("Error while acquiring database connection: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let dashboard = env.dashboards.get(dashboard_name)
        .ok_or(StatusCode::NOT_FOUND)?;

    let template = build_main(dashboard, &mut db, range_seconds)
        .await
        .map_err(|e| {
            println!("Error while building template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let html = template
        .render()
        .map_err(|e| {
            println!("Error while rendering template: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Html(html))
}

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

pub(crate) async fn build_main(config: &Dashboard, db: &mut SqliteConnection, range_seconds: u32) -> anyhow::Result<MainTemplate> {
    let mut widget_templates = vec![];
    for widget_config in config.widgets.iter() {
        let widget_template = widget_config.to_template(db, range_seconds).await?;
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

// async fn build_widget(config: Widget, db: &mut SqliteConnection, range_seconds: i64) -> anyhow::Result<WidgetTemplate>{
    
//     let color = config.stroke_css_color();

//     let template = match &config.typ {
//         WidgetType::Value{ series, label } => 
//         {
//             let data = db::get(db, &series, range_seconds).await?;
//             WidgetTemplateInner::Value(
//                 crate::view::ValueWidgetTemplate { 
//                     // config: config.clone(),
//                     color: color,
//                     label: label.clone(),
//                     point: data.last().map(|p| p.value) 
//                 }
//             )
//         },
//         WidgetType::Line{ series, label } => 
//         {
//             let data = db::get(db, &series, range_seconds).await?;
//             WidgetTemplateInner::Line(
//                 LineWidgetTemplate{
//                     // config: config.clone(),
//                     data: data,
//                     color: color,
//                     label: label.clone(),
//                     width: config.width,
//                     height: config.height,
//                 }
//             )   
//         },
//         WidgetType::Gauge{ series, min, max, label } => 
//         {
//             let data = db::get(db, &series, range_seconds).await?;
//             WidgetTemplateInner::Gauge(
//                 GaugeWidgetTemplate{
//                     // config: config.clone(),
//                     color: color,
//                     label: label.clone(),
//                     min: *min,
//                     max: *max,
//                     point: data.last().map(|p| p.value) 
//                 }
//             )
//         },
//         WidgetType::Label{ text } => 
//         {
//             WidgetTemplateInner::Label(
//                 LabelWidgetTemplate{
//                     // config: config.clone(),
//                     text: text.clone()
//                 }
//             )
//         },
//         WidgetType::Freshness{ series } => 
//         {
//             let data = db::get(db, &series, range_seconds).await?;
//             WidgetTemplateInner::Freshness(
//                 FreshnessWidgetTemplate{
//                     // config: config.clone(),
//                     last_update_time: data.last().map(|p| p.time) 
//                 }
//             )
//         },
//         WidgetType::Range{ range, label } => 
//         {
//             WidgetTemplateInner::Range(
//                 RangeWidgetTemplate{
//                     // config: config.clone(),
//                     range: range.unwrap_or(DEFAULT_RANGE_SECONDS),
//                     label: label.clone()
//                 }
//             )
//         },
//     };
//     Ok(
//         WidgetTemplate{
//             config,
//             template,
//         }
//     )
// }