use std::sync::Arc;

use ahash::RandomState;
use async_recursion::async_recursion;
use camino::Utf8Path;
use canary::{err, Channel, Result};
use compact_str::CompactString;
use dashmap::DashMap;
use serde_repr::{Deserialize_repr, Serialize_repr};

pub enum Service {
    Dynamic(Box<dyn Fn(Channel) + Send + Sync + 'static>),
    Static(fn(Channel)),
}

impl Service {
    pub(crate) fn call(&self, channel: Channel) {
        match self {
            Service::Dynamic(svc) => svc(channel),
            Service::Static(svc) => svc(channel),
        }
    }
}

impl<F, T> From<T> for Service
where
    F: std::future::Future<Output = Result<()>> + Send + Sync + 'static,
    T: Fn(Channel) -> F + 'static + Send + Sync,
{
    fn from(s: T) -> Self {
        Service::Dynamic(Box::new(move |c| {
            tokio::spawn(s(c));
        }))
    }
}

type Key = CompactString;
type InnerRoute = DashMap<Key, Storable, RandomState>;

#[derive(Default, Clone)]
pub struct Router {
    map: Flavor,
}

pub enum Flavor {
    Static(&'static InnerRoute),
    Arc(Arc<InnerRoute>),
}

impl Clone for Flavor {
    fn clone(&self) -> Self {
        match self {
            Self::Static(map) => Self::Static(map),
            Self::Arc(map) => Self::Arc(map.clone()),
        }
    }
}

impl Default for Flavor {
    fn default() -> Self {
        Flavor::Arc(Arc::new(InnerRoute::default()))
    }
}

impl std::ops::Deref for Flavor {
    type Target = InnerRoute;

    fn deref(&self) -> &Self::Target {
        match self {
            Flavor::Static(r) => r,
            Flavor::Arc(r) => r,
        }
    }
}

#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Status {
    Found,
    NotFound,
}

pub enum Storable {
    Svc(Service),
    Router(Router), // tree structure makes sure that arc cannot outlive inner, hence no possibility for memory leaks
}

impl Router {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn route(self, addr: impl Into<Key>, svc: impl Into<Service>) -> Self {
        self.map.insert(addr.into(), Storable::Svc(svc.into()));
        self
    }
    pub(crate) async fn insert(&self, mut c: Channel) -> Result<()> {
        let key: Key = c.receive().await?;
        let path = Utf8Path::new(key.as_str());
        let mut iter = path.into_iter();
        if let Err(_) = self.inner_switch(c, &mut iter, true).await {
            return err!((not_found, "route not found"));
        };
        Ok(())
    }
    #[async_recursion]
    async fn inner_switch(
        &self,
        mut c: Channel,
        at: &mut camino::Iter<'_>,
        discover: bool,
    ) -> Result<(), Channel> {
        let res = match at.next() {
            Some(key) => match self.map.get(key) {
                Some(storable) => match storable.value() {
                    Storable::Svc(svc) => {
                        if discover {
                            if let Ok(_) = c.send(Status::Found).await {
                                svc.call(c);
                            };
                        }
                        Ok(())
                    }
                    Storable::Router(router) => {
                        let fut = router.inner_switch(c, at, discover);
                        fut.await
                    }
                },
                None => Err(c),
            },
            None => Err(c),
        };
        if discover {
            if let Err(mut c) = res {
                c.send(Status::NotFound).await.ok();
                return Err(c);
            }
        }
        res
    }
}
