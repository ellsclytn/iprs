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
    fn usable_range(&self) -> Option<String>;
}

impl Ranges for Ipv4Net {
    fn addresses_in_network(&self) -> u32 {
        let prefix_len = self.prefix_len();
        2u32.saturating_pow(32 - prefix_len as u32)
    }

    fn network_range(&self) -> String {
        format!("{} - {}", self.network(), self.broadcast())
    }

    fn usable_range(&self) -> Option<String> {
        if self.prefix_len() > 30 {
            return None;
        }

        let first = self.network().to_u32().saturating_add(1);
        let last = self.broadcast().to_u32().saturating_sub(1);

        Some(format!("{} - {}", first.to_ipv4(), last.to_ipv4()))
    }
}

fn format_attribute<T>(name: &str, value: T) -> String
where
    T: fmt::Display,
{
    format!("{: <24}- {}", name, value)
}

impl Summary for Ipv4Net {
    fn summarize(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("-[ipv4 : {}] - 0\n\n[CIDR]", self));

        lines.push(format_attribute("Host address", self));
        lines.push(format_attribute("Host address (decimal)", self.addr().to_u32()));
        lines.push(format_attribute("Host address (hex)", format!("{:X}", self.addr().to_u32())));
        lines.push(format_attribute("Network address", self.network()));
        lines.push(format_attribute("Network mask", self.netmask()));
        lines.push(format_attribute("Network mask (bits)", self.prefix_len()));
        lines.push(format_attribute(
            "Network mask (hex)",
            format!("{:X}", self.netmask().to_u32()),
        ));
        lines.push(format_attribute("Broadcast address", self.broadcast()));
        lines.push(format_attribute("Cisco wildcard mask", (!self.netmask().to_u32()).to_ipv4()));
        lines.push(format_attribute("Addresses in network", self.addresses_in_network()));
        lines.push(format_attribute("Network range", self.network_range()));

        if let Some(usable_range) = self.usable_range() {
            lines.push(format_attribute("Usable range", usable_range));
        }

        lines.join("\n")
    }
}
