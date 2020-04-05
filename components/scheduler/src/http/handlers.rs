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
    // bench_server(&form["name"]);
    let server = flow::Server::new(form["name"].clone(), Default::default());
    scheduler.add_server(server);
    Ok(warp::reply::reply())
}

async fn scp(
    from_host: Option<&str>,
    from: &Path,
    to_host: Option<&str>,
    to: &Path,
) -> Result<(), Box<dyn Error>> {
    use tokio::process::Command;
    let from = if let Some(from_host) = from_host {
        format!("{}:{}", from_host, from.display())
    } else {
        from.display().to_string()
    };
    let to = if let Some(to_host) = to_host {
        format!("{}:{}", to_host, to.display())
    } else {
        to.display().to_string()
    };

    Command::new("scp").arg(from).arg(to).spawn()?.await?;
    Ok(())
}

async fn ssh(host: &str, cmd: &str, sudo_pass: &str) -> Result<String, Box<dyn Error>> {
    use tokio::process::Command;

    let res = Command::new("ssh")
        .arg(host)
        .arg(format!("'echo {} | sudo -S {}'", sudo_pass, cmd))
        .spawn()?
        .wait_with_output()
        .await?;

    Ok(String::from_utf8_lossy(&res.stdout).to_string())
}

pub async fn get_task(scheduler: Scheduler) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let source_template = include_str!("./pages/task.hbs");
    let scheduler = scheduler.lock().await;
    let mut map = HashMap::<&'static str, _>::new();
    map.insert("tasks", scheduler.get_tasks());

    let res = HBS.render_template(&source_template[..], &map).unwrap();
    Ok(warp::reply::html(res))
}

pub async fn post_task(
    scheduler: Scheduler,
    form: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::reject::Rejection> {
    let mut scheduler = scheduler.lock().await;
    let request = flow::ResourceProfile {
        cpu: form["cpu"].parse::<u8>().unwrap(),
        memory: form["memory"].parse::<u8>().unwrap(),
        disk: form["disk"].parse::<u8>().unwrap(),
        network: form["network"].parse::<u8>().unwrap(),
    };
    let task = flow::Task::new(form["name"].clone(), request);
    scheduler.add_task(task);
    Ok(warp::reply::reply())
}
