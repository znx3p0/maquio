





async fn main() -> Result<()> {
    let router = Router::new()
        .route("/", hello)
        .route("/hello", hello);
    let router = Arc::new(router);
    let tcp_handle = Tcp::bind("127.0.0.1:8080", router.clone()).await?;
    let wss_handle = Wss::bind("127.0.0.1:8081", router).await?;

    tcp_handle.await?;
    wss_handle.await
}

async fn hello(c: Channel, r: &Ctx) -> Result<()> {
    Ok(())
}

