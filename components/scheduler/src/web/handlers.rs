use super::graph::Graph;
use futures::{FutureExt, StreamExt};
use handlebars::Handlebars;
use lazy_static::lazy_static;
use pharos::*;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use warp::ws::{Message, WebSocket};

lazy_static! {
    static ref HBS: Handlebars<'static> = {
        let mut handlebars = Handlebars::new();
        handlebars
            .register_template_string("footer", include_str!("./pages/footer.hbs"))
            .unwrap();
        handlebars
            .register_template_string("header", include_str!("./pages/header.hbs"))
            .unwrap();
        handlebars
    };
}

pub async fn get_graph_html() -> Result<impl warp::Reply, warp::reject::Rejection> {
    let source_template = include_str!("./pages/graph.hbs");
    let res = HBS.render_template(&source_template[..], &{}).unwrap();

    Ok(warp::reply::html(res))
}

pub async fn graphflow(ws: WebSocket, graph: Arc<Mutex<Pharos<Graph>>>) {
    let (ws_tx, _) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.forward(ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
        }
    }));
    let mut events = graph
        .lock()
        .await
        .observe(Channel::Bounded(3).into())
        .expect("observe");
    log::debug!("Monitoring for events");

    loop {
        let evt = events.next().await.unwrap();
        log::debug!("New event detected");
        if let Err(_disconnected) = tx.send(Ok(Message::text(serde_json::to_string(&evt).unwrap())))
        {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
        }
    }
}

pub async fn get_index() -> Result<impl warp::Reply, warp::reject::Rejection> {
    let source_template = include_str!("./pages/index.hbs");
    let res = HBS.render_template(&source_template[..], &{}).unwrap();

    Ok(warp::reply::html(res))
}
