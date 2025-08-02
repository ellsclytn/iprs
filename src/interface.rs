pub mod ipv4;
pub mod ipv6;
pub mod summary;

use ipnet::IpNet;
pub use summary::Summary;

pub trait IpNetSummary {
    fn summarize(&self) -> String;
}

impl IpNetSummary for IpNet {
    fn summarize(&self) -> String {
        match self {
            IpNet::V4(ipv4) => ipv4.summarize(),
            IpNet::V6(ipv6) => ipv6.summarize(),
        }
    }
}
