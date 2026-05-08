mod ai;
mod game;
mod protocol;
mod server;

use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    println!("十二生肖游戏服务器启动中...");
    println!("请在浏览器中打开 http://localhost:{}", port);

    server::run_server(port).await;
}
