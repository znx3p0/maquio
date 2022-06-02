use std::{fmt::Display, str::FromStr};

use canary::{err, prelude::Addr, Channel, Error, Result};
use compact_str::CompactString;
use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

use crate::router::Status;

#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ServiceAddr {
    addr: Addr,
    id: CompactString,
}

impl ServiceAddr {
    pub fn new(f: impl AsRef<str>) -> Result<Self> {
        Self::from_str(f.as_ref())
    }
    pub async fn connect(&self) -> Result<Channel> {
        let mut chan = self.addr.connect().await?;
        chan.send(&self.id).await?;
        match chan.receive::<Status>().await? {
            Status::Found => Ok(chan),
            Status::NotFound => err!((not_found, "service id `{}` not found", self.id)),
        }
    }
    pub fn addr(&self) -> &Addr {
        &self.addr
    }
    pub fn id(&self) -> &CompactString {
        &self.id
    }
    pub fn inner(self) -> (Addr, CompactString) {
        (self.addr, self.id)
    }
}

impl FromStr for ServiceAddr {
    type Err = Error;

    /// service://tcp@127.0.0.1:8080
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let (id, addr) = string
            .rsplit_once("://")
            .ok_or(err!(invalid_input, "malformed service address"))?;
        let id = CompactString::new(id);
        let addr = Addr::from_str(addr)?;
        Ok(ServiceAddr { addr, id })
    }
}
impl Display for ServiceAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}://{}", self.id, &self.addr)
    }
}

impl std::fmt::Debug for ServiceAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Serialize for ServiceAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            self.to_string().serialize(serializer)
        } else {
            let mut ser = serializer.serialize_seq(Some(2))?;
            ser.serialize_element(&self.id)?;
            ser.serialize_element(&self.addr)?;
            ser.end()
        }
    }
}

impl<'de> Deserialize<'de> for ServiceAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let string = CompactString::deserialize(deserializer)?;
            ServiceAddr::from_str(&string).map_err(serde::de::Error::custom)
        } else {
            struct ServiceAddrVisitor;
            impl<'de> Visitor<'de> for ServiceAddrVisitor {
                type Value = ServiceAddr;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, "ServiceAddr")
                }
                fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where
                    A: serde::de::SeqAccess<'de>,
                {
                    let id = seq
                        .next_element::<CompactString>()?
                        .ok_or(serde::de::Error::custom("expected Id, found nothing"))?;
                    let addr = seq
                        .next_element::<Addr>()?
                        .ok_or(serde::de::Error::custom("expected Addr, found nothing"))?;
                    Ok(ServiceAddr { id, addr })
                }
            }
            let visitor = ServiceAddrVisitor;
            deserializer.deserialize_seq(visitor)
        }
    }
}
