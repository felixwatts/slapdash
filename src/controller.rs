use std::num::FpCategory;
use sqlx::SqliteConnection;
use crate::db;
use axum::extract::{Path, State};
use crate::model::WidgetType;
use crate::view::{FreshnessWidgetTemplate, GaugeWidgetTemplate, LineWidgetTemplate, WidgetTemplateInner};
use crate::{model::{grid_height, grid_width, Widget}, view::{MainTemplate, WidgetTemplate}};
use axum::http::StatusCode;
use askama::Template;
use axum::response::Html;
use crate::env::Environment;

pub(crate) async fn get_default (
    State(env): State<Environment>,
) -> Result<Html<String>, StatusCode>
{
    get_dashboard("default", &env).await
}

pub(crate) async fn get (
    Path(dashboard): Path<String>, 
    State(env): State<Environment>,
) -> Result<Html<String>, StatusCode>
{
    get_dashboard(&dashboard, &env).await
}

pub(crate) async fn get_widget (
    Path((dashboard, index)): Path<(String, usize)>,
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

    render(&dashboard.name, vec![(index, widget)], &env).await
}

async fn get_dashboard(dashboard_name: &str, env: &Environment) -> Result<Html<String>, StatusCode> {
    let dashboard = env.dashboards.get(dashboard_name)
        .ok_or(StatusCode::NOT_FOUND)?;

    let widgets = dashboard.widgets.iter().cloned().enumerate().collect();

    render(&dashboard.name, widgets, env).await
}

/// Render a page of the given widgets, each paired with its index in the named dashboard.
async fn render(dashboard_name: &str, widgets: Vec<(usize, Widget)>, env: &Environment) -> Result<Html<String>, StatusCode> {
    let mut db = env
        .db
        .acquire()
        .await
        .map_err(|e| {
            println!("Error while acquiring database connection: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let template = build_main(dashboard_name, widgets, &mut db)
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

pub(crate) async fn build_main(dashboard_name: &str, widgets: Vec<(usize, Widget)>, db: &mut SqliteConnection) -> anyhow::Result<MainTemplate> {
    let configs: Vec<Widget> = widgets.iter().map(|(_, w)| w.clone()).collect();

    let mut widget_templates = vec![];
    for (index, widget_config) in widgets {
        let href = format!("/{dashboard_name}/{index}");
        let widget_template = build_widget(widget_config, href, db).await?;
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

async fn build_widget(config: Widget, href: String, db: &mut SqliteConnection) -> anyhow::Result<WidgetTemplate>{
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
            href,
            template,
        }
    )
}