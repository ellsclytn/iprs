use ipnet::Ipv6Net;
use std::{io::Write, net::Ipv6Addr};

use crate::{
    context::Ctx,
    error::Result,
    interface::{traits::*, Interface},
    rng::DefaultRng,
};

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

impl NetworkCore for Ipv6Net {
    type Address = Ipv6Addr;
    type Primitive = u128;

    fn addr(&self) -> Self::Address {
        self.addr()
    }

    fn prefix_len(&self) -> u8 {
        self.prefix_len()
    }

    fn broadcast(&self) -> Self::Address {
        self.broadcast()
    }

    fn subnets(&self, prefix_len: u8) -> Result<impl Iterator<Item = Self>> {
        Ok(self.subnets(prefix_len)?)
    }
}

impl NetworkDisplay for Ipv6Net {
    const IP_VERSION: &'static str = "ipv6";
    const FORMAT_WIDTH: usize = 39;

    fn from_addr_prefix(addr: Self::Address, prefix: u8) -> Result<Self> {
        Ok(Ipv6Net::new(addr, prefix)?)
    }
}

impl<W: Write, E: Write> NetworkSummarize<W, E> for Ipv6Net {
    fn summarize(&self, ctx: &mut Ctx<W, E>) -> Result<()> {
        ctx.writeln(format!("-[ipv6 : {self}] - 0\n"))?;
        ctx.writeln("[IPV6 INFO]".to_string())?;

        ctx.writeln(Self::format_attribute(
            "Expanded Address",
            self.expanded_address(),
        ))?;
        ctx.writeln(Self::format_attribute("Compressed Address", self.addr()))?;
        ctx.writeln(Self::format_attribute(
            "Subnet Prefix (masked)",
            self.subnet_prefix_masked(),
        ))?;
        ctx.writeln(Self::format_attribute(
            "Address ID (masked)",
            self.address_id_masked(),
        ))?;
        ctx.writeln(Self::format_attribute(
            "Prefix address",
            self.trunc().netmask(),
        ))?;
        ctx.writeln(Self::format_attribute("Prefix length", self.prefix_len()))?;
        ctx.writeln(Self::format_attribute("Address type", self.address_type()))?;

        let network_range_start = self.trunc().network();
        let network_range_end = self.trunc().broadcast();
        ctx.writeln(format!("{: <24}- {network_range_start} -", "Network range"))?;
        ctx.writeln(format!("{: <25} {}", " ", network_range_end))?;

        Ok(())
    }
}

impl Interface for Ipv6Net {
    fn summarize<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>) -> Result<()> {
        NetworkSummarize::summarize(self, ctx)
    }

    fn split<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>, mask: u8) -> Result<()> {
        NetworkDisplay::split(self, ctx, mask)
    }

    fn random_split<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>, split: u8) -> Result<()> {
        let mut rng = DefaultRng;
        NetworkDisplay::summarize_random_split(self, ctx, split, &mut rng)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::context::test_util::{create_test_ctx, get_output_as_string};
    use crate::rng::RandomRangeGenerator;
    use crate::Error;
    use pretty_assertions::assert_eq;

    fn ip_to_u128(ip: &str) -> u128 {
        let cidr = format!("{ip}/128");

        u128::from(Ipv6Net::from_str(&cidr).unwrap().addr())
    }

    struct MockRng {
        value: u128,
    }

    impl MockRng {
        fn new(value: u128) -> Self {
            Self { value }
        }
    }

    impl RandomRangeGenerator<u128> for MockRng {
        fn random_range(&mut self, _range: std::ops::Range<u128>) -> u128 {
            self.value
        }
    }

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

        Interface::summarize(&ip, &mut ctx).unwrap();
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

        Interface::split(&ip, &mut ctx, 84).unwrap();
        let output = get_output_as_string(&ctx);

        assert_eq!(output, expected);
    }

    #[test]
    fn reports_oversized_range_split() {
        let ip = Ipv6Net::from_str("1234:5678::/64").unwrap();
        let mut ctx = create_test_ctx();

        let e = Interface::split(&ip, &mut ctx, 25).unwrap_err();

        assert!(matches!(e, Error::SplitSmallerThanPrefixLen(25, 64)));
    }

    #[test]
    fn random_split_produces_different_results_with_different_random_values() {
        let ip = Ipv6Net::from_str("7a18:549a:ecb0:5573:edfc:fa96:d303:ce5b/48").unwrap();

        let mut mock_rng1 = MockRng::new(ip_to_u128("7a18:549a:ecb0:5573:edfc:fa96:d303:ce5b"));
        let output1 = ip.generate_random_split(64, &mut mock_rng1).unwrap();

        assert_eq!(output1.to_string(), "7a18:549a:ecb0:5573::/64");

        let mut mock_rng2 = MockRng::new(ip_to_u128("45c0:b8e4:e243:4159:439c:d13d:3e2a:8c80"));
        let output2 = ip.generate_random_split(72, &mut mock_rng2).unwrap();

        assert_eq!(output2.to_string(), "7a18:549a:ecb0:4159:4300::/72");

        assert_ne!(output1, output2);
    }

    #[test]
    fn random_split_works_with_netmask_0_input() {
        let ip = Ipv6Net::from_str("::/0").unwrap();

        let mut mock_rng = MockRng::new(ip_to_u128("4cc7:8e7:b232:e2dd:4920:68b5:e628:406f"));
        let output = ip.generate_random_split(64, &mut mock_rng).unwrap();

        assert_eq!(output.to_string(), "4cc7:8e7:b232:e2dd::/64");
    }
}
