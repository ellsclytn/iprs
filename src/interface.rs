pub mod ipv4;
pub mod ipv6;

use ipnet::IpNet;

pub trait Interface {
    fn summarize(&self) -> String;
    fn split(&self, split: u8) -> String;
}

impl Interface for IpNet {
    fn summarize(&self) -> String {
        match self {
            IpNet::V4(ipv4) => ipv4.summarize(),
            IpNet::V6(ipv6) => ipv6.summarize(),
        }
    }

    fn split(&self, mask: u8) -> String {
        match self {
            IpNet::V4(ipv4) => ipv4.split(mask),
            IpNet::V6(ipv6) => ipv6.split(mask),
        }
    }
}
