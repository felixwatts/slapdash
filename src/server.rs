use crate::env::Dashboards;
use axum::{Router, routing::get};
use crate::env::Environment;
use crate::controller;
use std::net::SocketAddr;
use crate::env::Settings;

pub struct Server;

impl Server{
    pub async fn serve(listen_addr: &Option<SocketAddr>, secret: &Option<String>) -> anyhow::Result<()> {
        let (dashboards_changed_tx, mut dashboards_changed_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
        let watcher = watchr_filesystem::FileWatcher::new(vec![
            Dashboards::path()?
        ]);
        tokio::spawn(async move {
            let _ = watcher.watch(move |_| { 
                let _ = dashboards_changed_tx.send(());
            }).await;
        });

        loop{
            let env = Environment::load().await?;
            let secret = secret.as_ref().unwrap_or(&env.settings.secret).to_string();
            let listen_addr = listen_addr.unwrap_or(env.settings.listen_addr);
            let dashboard_list = env.dashboards.list();
            let app = Router::new()
                .route("/", get(controller::get_default))
                .route("/{dashboard}", get(controller::get))
                .route("/{secret}/{series}/{value}", get(controller::put))
                .with_state(env);
            let listener = tokio::net::TcpListener::bind(listen_addr).await?;
        
            println!("Serving at: http://{listen_addr}/(<dashboard>)");
            println!("Dashboards:\n\t{}", &dashboard_list.join("\n\t"));
            println!("Push data: GET http://{}/{}/<series>/<value>", listen_addr, &secret);

            tokio::select! {
                _ = dashboards_changed_rx.recv() => {}
                _ = axum::serve(listener, app) => {}
            }

            println!("Dashboards changed, reloading..");
        }
    }
}

