use std::{fmt, net::Ipv4Addr};

use crate::interface::Summary;
use ipnet::Ipv4Net;

trait ToPrimitive {
    fn to_u32(&self) -> u32;
}

impl ToPrimitive for Ipv4Addr {
    fn to_u32(&self) -> u32 {
        (*self).into()
    }
}

trait ToIpv4 {
    fn to_ipv4(&self) -> Ipv4Addr;
}

impl ToIpv4 for u32 {
    fn to_ipv4(&self) -> Ipv4Addr {
        (*self).into()
    }
}

trait Ranges {
    fn addresses_in_network(&self) -> u32;
    fn network_range(&self) -> String;
    fn usable_range(&self) -> String;
}

impl Ranges for Ipv4Net {
    fn addresses_in_network(&self) -> u32 {
        let prefix_len = self.prefix_len();
        2u32.saturating_pow(32 - prefix_len as u32)
    }

    fn network_range(&self) -> String {
        format!("{} - {}", self.network(), self.broadcast())
    }

    fn usable_range(&self) -> String {
        let first = self.network().to_u32().saturating_add(1);
        let last = self.broadcast().to_u32().saturating_sub(1);

        format!("{} - {}", first.to_ipv4(), last.to_ipv4())
    }
}

fn print_attribute<T>(name: &str, value: T)
where
    T: fmt::Display,
{
    println!("{: <24}- {}", name, value);
}

impl Summary for Ipv4Net {
    fn print_summary(&self) {
        println!("-[ipv4 : {}] - 0\n", self);
        println!("[CIDR]");

        print_attribute("Host address", self);
        print_attribute("Host address (decimal)", self.addr().to_u32());
        print_attribute("Host address (hex)", format!("{:X}", self.addr().to_u32()));
        print_attribute("Network address", self.network());
        print_attribute("Network mask", self.netmask());
        print_attribute("Network mask (bits)", self.prefix_len());
        print_attribute(
            "Network mask (hex)",
            format!("{:X}", self.netmask().to_u32()),
        );
        print_attribute("Broadcast address", self.broadcast());
        print_attribute("Cisco wildcard mask", (!self.netmask().to_u32()).to_ipv4());
        print_attribute("Addresses in network", self.addresses_in_network());
        print_attribute("Network range", self.network_range());
        print_attribute("Usable range", self.usable_range());
    }
}
