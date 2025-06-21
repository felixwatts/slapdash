use std::num::FpCategory;

use sqlx::{Acquire, PgConnection, Postgres};
use tide::{Response, StatusCode};
use tide_sqlx::SQLxRequestExt;
use crate::db;

use crate::model::WidgetType;
use crate::view::{FreshnessWidgetTemplate, GagueWidgetTemplate, LineWidgetTemplate, WidgetTemplateInner};
use crate::{model::{Dashboard, Widget}, view::{MainTemplate, WidgetTemplate}};

pub(crate) async fn get(req: tide::Request<(String, Dashboard)>) -> tide::Result {
    let mut db = req.sqlx_conn::<Postgres>().await;
    let (_secret, config) = req.state();

    let template = build_main(config, db.acquire().await?).await?;

    Ok(askama_tide::into_response(&template))
}

pub(crate) async fn put(req: tide::Request<(String, Dashboard)>) -> tide::Result {
    let (expected_secret, _config) = req.state();
    let actual_secret = req.param("secret")?;

    if actual_secret != expected_secret {
        return Err(tide::Error::from_str(StatusCode::Unauthorized, "Unauthorized"));
    }
    let series = req.param("series")?;
    let value_str = req.param("value")?;
    let value: f32 = value_str.parse()?;

    match value.classify() {
        FpCategory::Normal | FpCategory::Zero => {
            let mut db = req.sqlx_conn::<Postgres>().await;
            db::put(db.acquire().await?, series, value).await.map_err(|msg| tide::Error::from_str(StatusCode::InternalServerError, msg))?;
        },
        _ => {}
    }

    Ok(Response::builder(StatusCode::Ok).build())
}

pub(crate) async fn build_main(config: &Dashboard, db: &mut PgConnection) -> tide::Result<MainTemplate> {
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

async fn build_widget(config: Widget, db: &mut PgConnection) -> tide::Result<WidgetTemplate>{
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
        WidgetType::Gague{ .. } => WidgetTemplateInner::Gague(
            GagueWidgetTemplate{
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