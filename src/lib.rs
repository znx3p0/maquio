pub mod providers;
pub mod router;
mod service_addr;

pub use router::Router;

pub use service_addr::ServiceAddr;

pub use canary;

pub use canary::err;
pub use canary::Channel;
pub use canary::Error;
pub use canary::Result;
