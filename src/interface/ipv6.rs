use std::fmt;

use crate::interface::Summary;
use ipnet::{IpBitAnd, Ipv6Net};

trait PrintableProperties {
    fn expanded_address(&self) -> String;
    fn subnet_prefix_masked(&self) -> String;
    fn address_id_masked(&self) -> String;
    fn address_type(&self) -> &str;
}

impl PrintableProperties for Ipv6Net {
    fn expanded_address(&self) -> String {
        let addr = self.addr();
        format!(
            "{:04x}:{:04x}:{:04x}:{:04x}:{:04x}:{:04x}:{:04x}:{:04x}",
            addr.segments()[0],
            addr.segments()[1],
            addr.segments()[2],
            addr.segments()[3],
            addr.segments()[4],
            addr.segments()[5],
            addr.segments()[6],
            addr.segments()[7]
        )
    }

    fn subnet_prefix_masked(&self) -> String {
        let id = self.netmask() & self.addr();

        format!("{id}/{}", self.prefix_len())
    }

    fn address_id_masked(&self) -> String {
        let id = self.hostmask() & self.addr();

        format!("{id}/{}", self.prefix_len())
    }

    fn address_type(&self) -> &str {
        let first_segment = self.addr().segments()[0];

        // https://www.iana.org/assignments/ipv6-address-space/ipv6-address-space.xhtml

        // 2000::/3 RFC4291 & RFC3513
        if (first_segment & 0xe000) == 0x2000 {
            return "Aggregatable Global Unicast Addresses";
        // fc00::/7 RFC4193
        } else if (first_segment & 0xfe00) == 0xfc00 {
            return "Unique Local Unicast";
        }

        todo!()
    }
}

fn format_attribute<T>(name: &str, value: T) -> String
where
    T: fmt::Display,
{
    format!("{: <24}- {}", name, value)
}

impl Summary for Ipv6Net {
    fn summarize(&self) -> String {
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!("-[ipv6 : {self}] - 0\n"));
        lines.push("[IPV6 INFO]".to_string());

        lines.push(format_attribute(
            "Expanded Address",
            self.expanded_address(),
        ));
        lines.push(format_attribute("Compressed Address", self.addr()));
        lines.push(format_attribute(
            "Subnet Prefix (masked)",
            self.subnet_prefix_masked(),
        ));
        lines.push(format_attribute(
            "Address ID (masked)",
            self.address_id_masked(),
        ));
        lines.push(format_attribute("Prefix address", self.trunc().netmask()));
        lines.push(format_attribute("Prefix length", self.prefix_len()));
        lines.push(format_attribute("Address type", self.address_type()));

        let network_range_start = self.trunc().network();
        let network_range_end = self.trunc().broadcast();
        lines.push(format!("{: <24}- {network_range_start} -", "Network range"));
        lines.push(format!("{: <25} {}", " ", network_range_end));

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn sumarizes_an_interface() {
        let expected = "-[ipv6 : 3bc7:a1c8:8d4:f9fc:3ed1:bfed:f539:a271/64] - 0

[IPV6 INFO]
Expanded Address        - 3bc7:a1c8:08d4:f9fc:3ed1:bfed:f539:a271
Compressed Address      - 3bc7:a1c8:8d4:f9fc:3ed1:bfed:f539:a271
Subnet Prefix (masked)  - 3bc7:a1c8:8d4:f9fc::/64
Address ID (masked)     - ::3ed1:bfed:f539:a271/64
Prefix address          - ffff:ffff:ffff:ffff::
Prefix length           - 64
Address type            - Aggregatable Global Unicast Addresses
Network range           - 3bc7:a1c8:8d4:f9fc:: -
                          3bc7:a1c8:8d4:f9fc:ffff:ffff:ffff:ffff";
        let ip = Ipv6Net::from_str("3bc7:a1c8:8d4:f9fc:3ed1:bfed:f539:a271/64").unwrap();

        assert_eq!(ip.summarize(), expected)
    }
}
