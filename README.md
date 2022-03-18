# Maquio

Library for building composable distributed systems.

```rust
async fn main() -> Result<()> {
    let router = Router::new()
        .route("/", hello)
        .route("/hello", hello);
    let tcp_handle = Tcp::bind("127.0.0.1:8080", router).await?;

    tcp_handle.await?;
    wss_handle.await
}

async fn hello(c: Channel) -> Result<()> {
    c.send("Hello world!");
    Ok(())
}
```
