use super::graph::Graph;
use super::Scheduler;
use super::SchedulerSubscription;
use crate::flow;
use crate::import::*;
use futures::{FutureExt, StreamExt};
use handlebars::Handlebars;
use lazy_static::lazy_static;
use tokio::sync::mpsc;
use warp::ws::{Message, WebSocket};

lazy_static! {
    static ref HBS: Handlebars<'static> = {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("footer", include_str!("./pages/footer.hbs")).unwrap();
        handlebars.register_template_string("header", include_str!("./pages/header.hbs")).unwrap();
        handlebars
    };
}

pub async fn get_graph_html() -> Result<impl warp::Reply, warp::reject::Rejection> {
    let source_template = include_str!("./pages/graph.hbs");
    let res = HBS.render_template(&source_template[..], &{}).unwrap();

    Ok(warp::reply::html(res))
}

pub async fn graphflow(ws: WebSocket, mut graph_rx: SchedulerSubscription) {
    let (ws_tx, _) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::task::spawn(rx.forward(ws_tx).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
        }
    }));

    let mut flows = graph_rx.recv().await.unwrap();

    loop {
        let graph = Graph::from_flow(flows);
        if let Err(_disconnected) =
            tx.send(Ok(Message::text(serde_json::to_string(&graph).unwrap())))
        {
            break;
        }
        flows = graph_rx.recv().await.unwrap();
        log::debug!("New event detected");
    }
}

pub async fn get_index() -> Result<impl warp::Reply, warp::reject::Rejection> {
    let source_template = include_str!("./pages/index.hbs");
    let res = HBS.render_template(&source_template[..], &{}).unwrap();

    Ok(warp::reply::html(res))
}

pub async fn get_server(scheduler: Scheduler) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let source_template = include_str!("./pages/server.hbs");
    let scheduler = scheduler.lock().await;
    let mut map = HashMap::<&'static str, _>::new();
    map.insert("servers", scheduler.get_servers());

    let res = HBS.render_template(&source_template[..], &map).unwrap();
    Ok(warp::reply::html(res))
}

pub async fn post_server(
    scheduler: Scheduler,
    form: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut scheduler = scheduler.lock().await;
    let server = flow::Server::new(form["name"].clone(), Default::default());
    scheduler.add_server(server);
    Ok(warp::reply::reply())
}
