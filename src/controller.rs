use std::num::FpCategory;
use sqlx::SqliteConnection;
use crate::db;
use axum::extract::{Path, Query, State};
use crate::model::{grid_height, grid_width, Widget, WidgetType};
use crate::view::MainTemplate;
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
    get_dashboard("default", &env, query.range).await
}

pub(crate) async fn get (
    Path(dashboard): Path<String>,
    Query(query): Query<DashboardQuery>,
    State(env): State<Environment>,
) -> Result<Html<String>, StatusCode>
{
    get_dashboard(&dashboard, &env, query.range).await
}

pub(crate) async fn get_widget (
    Path((dashboard, index)): Path<(String, usize)>,
    Query(query): Query<DashboardQuery>,
    State(env): State<Environment>,
) -> Result<Html<String>, StatusCode>
{
    let dashboard = env.dashboards.get(&dashboard)
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut widget = dashboard.widgets.get(index)
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();
    widget.left = 1;
    widget.top = 1;

    render(&dashboard.name, vec![(index, widget)], &env, query.range).await
}

fn resolve_range(range: Option<u32>) -> Result<u32, StatusCode> {
    match range {
        None => Ok(DEFAULT_RANGE_SECONDS),
        Some(r) if r > 0 => Ok(r),
        Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn get_dashboard(dashboard_name: &str, env: &Environment, range: Option<u32>) -> Result<Html<String>, StatusCode> {
    let dashboard = env.dashboards.get(dashboard_name)
        .ok_or(StatusCode::NOT_FOUND)?;

    let widgets = dashboard.widgets.iter().cloned().enumerate().collect();

    render(&dashboard.name, widgets, env, range).await
}

/// Render a page of the given widgets, each paired with its index in the named dashboard.
async fn render(dashboard_name: &str, widgets: Vec<(usize, Widget)>, env: &Environment, range: Option<u32>) -> Result<Html<String>, StatusCode> {
    let range_seconds = resolve_range(range)?;

    let mut db = env
        .db
        .acquire()
        .await
        .map_err(|e| {
            println!("Error while acquiring database connection: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let template = build_main(dashboard_name, widgets, &mut db, range_seconds, range)
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

/// `range` is the range explicitly requested by the URL, if any; it is carried through to widget links.
pub(crate) async fn build_main(
    dashboard_name: &str,
    widgets: Vec<(usize, Widget)>,
    db: &mut SqliteConnection,
    range_seconds: u32,
    range: Option<u32>,
) -> anyhow::Result<MainTemplate> {
    let configs: Vec<Widget> = widgets.iter().map(|(_, w)| w.clone()).collect();

    let mut widget_templates = vec![];
    for (index, widget_config) in widgets {
        let href = match widget_config.typ {
            // The range widget is already a link; nesting links is invalid HTML
            WidgetType::Range{ .. } => None,
            _ => Some(match range {
                Some(range) => format!("/{dashboard_name}/{index}?range={range}"),
                None => format!("/{dashboard_name}/{index}"),
            }),
        };
        let widget_template = widget_config.to_template(db, range_seconds, href).await?;
        widget_templates.push(widget_template)
    }

    Ok(
        MainTemplate{
            name: dashboard_name.to_string(),
            width: grid_width(&configs),
            height: grid_height(&configs),
            widgets: widget_templates
        }
    )
}
