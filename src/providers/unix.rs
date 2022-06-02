use std::path::Path;

use canary::providers::Unix as CUnix;
use canary::{err, Channel, Result};
use tokio::task::JoinHandle;

use crate::router::Status;
use crate::Router;

pub struct Unix;
impl Unix {
    pub async fn bind(addrs: impl AsRef<Path>, r: Router) -> Result<JoinHandle<Result<()>>> {
        let tcp = CUnix::bind(addrs).await?;
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
    pub async fn connect(addr: impl AsRef<Path> + std::fmt::Debug, id: &str) -> Result<Channel> {
        let mut c = CUnix::connect(addr).await?.encrypted().await?;
        c.send(id).await?;
        match c.receive::<Status>().await? {
            Status::Found => Ok(c),
            Status::NotFound => err!((not_found, "service id: `{}` not found", id)),
        }
    }
}
