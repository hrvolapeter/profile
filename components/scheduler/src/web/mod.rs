pub mod graph;
mod handlers;

use self::graph::Graph;
use warp::Filter;
use tokio::sync::watch::Receiver;

pub async fn serve(graph: Receiver<Graph>) {
    let routes = get_schedule_graph()
        .or(get_index())
        .or(get_api_schedule_graph(graph));
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}

fn get_schedule_graph() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::get()
        .and(warp::path!("schedule" / "graph"))
        .and_then(handlers::get_graph_html)
}

fn get_api_schedule_graph(
    graph: Receiver<Graph>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let graph = warp::any().map(move || graph.clone());
    warp::path!("api" / "schedule" / "graph")
        .and(warp::ws())
        .and(graph)
        .map(|ws: warp::ws::Ws, graph| {
            ws.on_upgrade(move |socket| handlers::graphflow(socket, graph))
        })
}

fn get_index() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path::end())
        .and_then(handlers::get_index)
}
