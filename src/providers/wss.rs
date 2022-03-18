
use canary::providers::Wss as CWss;
use canary::{Result, Channel, err};
use tokio::net::ToSocketAddrs;
use tokio::task::JoinHandle;

use crate::Router;
use crate::router::Status;

pub struct Wss;
impl Wss {
    pub async fn bind(addrs: impl ToSocketAddrs, r: Router) -> Result<JoinHandle<Result<()>>> {
        let tcp = CWss::bind(addrs).await?;
        Ok(tokio::spawn(async move {
            loop {
                let c = tcp.next().await?;
                let r = r.clone();
                tokio::task::spawn(async move {
                    let chan = c.encrypted().await?;
                    r.insert(chan).await?;
                    Ok::<_, canary::Error>(())
                });
            }
        }))
    }
    pub async fn connect(addr: impl ToSocketAddrs + std::fmt::Debug, id: &str) -> Result<Channel> {
        let mut c = CWss::connect(addr).await?.encrypted().await?;
        c.send(id).await?;
        match c.receive::<Status>().await? {
            Status::Found => Ok(c),
            Status::NotFound => err!((format!("service id: `{id}` not found"))),
        }
    }
}