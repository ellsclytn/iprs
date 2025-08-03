use std::{
    fmt,
    io::{self, Write},
};

use crate::{context::Ctx, interface::Interface};
use ipnet::Ipv6Net;

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
        // fe80://10 RFC4291 & RFC3513
        } else if (first_segment & 0xffc0) == 0xfe80 {
            return "Link-Scoped Unicast";
        // ff00::/8 RFC4291 & RFC3513
        } else if (first_segment & 0xff00) == 0xff00 {
            return "Multicast";
        }

        "Reserved by IETF"
    }
}

fn format_attribute<T>(name: &str, value: T) -> String
where
    T: fmt::Display,
{
    format!("{name: <24}- {value}")
}

impl Interface for Ipv6Net {
    fn summarize<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>) -> Result<(), io::Error> {
        ctx.writeln(format!("-[ipv6 : {self}] - 0\n"))?;
        ctx.writeln("[IPV6 INFO]".to_string())?;

        ctx.writeln(format_attribute(
            "Expanded Address",
            self.expanded_address(),
        ))?;
        ctx.writeln(format_attribute("Compressed Address", self.addr()))?;
        ctx.writeln(format_attribute(
            "Subnet Prefix (masked)",
            self.subnet_prefix_masked(),
        ))?;
        ctx.writeln(format_attribute(
            "Address ID (masked)",
            self.address_id_masked(),
        ))?;
        ctx.writeln(format_attribute("Prefix address", self.trunc().netmask()))?;
        ctx.writeln(format_attribute("Prefix length", self.prefix_len()))?;
        ctx.writeln(format_attribute("Address type", self.address_type()))?;

        let network_range_start = self.trunc().network();
        let network_range_end = self.trunc().broadcast();
        ctx.writeln(format!("{: <24}- {network_range_start} -", "Network range"))?;
        ctx.writeln(format!("{: <25} {}", " ", network_range_end))?;

        Ok(())
    }

    fn split<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>, mask: u8) -> Result<(), io::Error> {
        ctx.writeln(format!("-[ipv6 : {self}] - 0\n"))?;
        ctx.writeln("[Split network]".to_string())?;

        match self.subnets(mask) {
            Ok(subnets) => {
                for subnet in subnets {
                    ctx.writeln(format!(
                        "Network - {:<39} - {}",
                        subnet.addr(),
                        subnet.broadcast()
                    ))?;
                }
            }
            Err(_) => {
                ctx.writeln("-[ERR : Oversized splitmask]".to_string())?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::context::test_util::{create_test_ctx, get_output_as_string};
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
                          3bc7:a1c8:8d4:f9fc:ffff:ffff:ffff:ffff
";
        let ip = Ipv6Net::from_str("3bc7:a1c8:8d4:f9fc:3ed1:bfed:f539:a271/64").unwrap();
        let mut ctx = create_test_ctx();

        ip.summarize(&mut ctx).unwrap();
        let output = get_output_as_string(&ctx);

        assert_eq!(output, expected)
    }

    #[test]
    fn splits_a_range() {
        let expected = "-[ipv6 : ffff::/81] - 0

[Split network]
Network - ffff::                                  - ffff::fff:ffff:ffff
Network - ffff::1000:0:0                          - ffff::1fff:ffff:ffff
Network - ffff::2000:0:0                          - ffff::2fff:ffff:ffff
Network - ffff::3000:0:0                          - ffff::3fff:ffff:ffff
Network - ffff::4000:0:0                          - ffff::4fff:ffff:ffff
Network - ffff::5000:0:0                          - ffff::5fff:ffff:ffff
Network - ffff::6000:0:0                          - ffff::6fff:ffff:ffff
Network - ffff::7000:0:0                          - ffff::7fff:ffff:ffff
";
        let ip = Ipv6Net::from_str("ffff::/81").unwrap();
        let mut ctx = create_test_ctx();

        ip.split(&mut ctx, 84).unwrap();
        let output = get_output_as_string(&ctx);

        assert_eq!(output, expected);
    }

    #[test]
    fn reports_oversized_range_split() {
        let expected = "-[ipv6 : 1234:5678::/64] - 0

[Split network]
-[ERR : Oversized splitmask]
";
        let ip = Ipv6Net::from_str("1234:5678::/64").unwrap();
        let mut ctx = create_test_ctx();

        ip.split(&mut ctx, 25).unwrap();
        let output = get_output_as_string(&ctx);

        assert_eq!(output, expected);
    }
}
