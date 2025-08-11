use std::{io::Write, net::Ipv4Addr};

use crate::context::Ctx;
use crate::error::Result;
use crate::interface::{traits::*, Interface};
use crate::rng::DefaultRng;
use ipnet::Ipv4Net;

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

        let first = u32::from(self.network()).saturating_add(1);
        let last = u32::from(self.broadcast()).saturating_sub(1);

        Some(format!(
            "{} - {}",
            Ipv4Addr::from(first),
            Ipv4Addr::from(last)
        ))
    }
}

impl NetworkCore for Ipv4Net {
    type Address = Ipv4Addr;
    type Primitive = u32;

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

impl NetworkDisplay for Ipv4Net {
    const IP_VERSION: &'static str = "ipv4";
    const FORMAT_WIDTH: usize = 15;

    fn from_addr_prefix(addr: Self::Address, prefix: u8) -> Result<Self> {
        Ok(Ipv4Net::new(addr, prefix)?)
    }
}

impl<W: Write, E: Write> NetworkSummarize<W, E> for Ipv4Net {
    fn summarize(&self, ctx: &mut Ctx<W, E>) -> Result<()> {
        ctx.writeln(format!("-[ipv4 : {self}] - 0\n\n[CIDR]"))?;

        ctx.writeln(Self::format_attribute("Host address", self.addr()))?;
        ctx.writeln(Self::format_attribute(
            "Host address (decimal)",
            u32::from(self.addr()),
        ))?;
        ctx.writeln(Self::format_attribute(
            "Host address (hex)",
            format!("{:X}", u32::from(self.addr())),
        ))?;
        ctx.writeln(Self::format_attribute("Network address", self.network()))?;
        ctx.writeln(Self::format_attribute("Network mask", self.netmask()))?;
        ctx.writeln(Self::format_attribute(
            "Network mask (bits)",
            self.prefix_len(),
        ))?;
        ctx.writeln(Self::format_attribute(
            "Network mask (hex)",
            format!("{:X}", u32::from(self.netmask())),
        ))?;
        ctx.writeln(Self::format_attribute(
            "Broadcast address",
            self.broadcast(),
        ))?;
        ctx.writeln(Self::format_attribute("Cisco wildcard", !self.netmask()))?;
        ctx.writeln(Self::format_attribute(
            "Addresses in network",
            self.addresses_in_network(),
        ))?;
        ctx.writeln(Self::format_attribute(
            "Network range",
            self.network_range(),
        ))?;

        if let Some(usable_range) = self.usable_range() {
            ctx.writeln(Self::format_attribute("Usable range", usable_range))?;
        }

        Ok(())
    }
}

impl Interface for Ipv4Net {
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

    fn ip_to_u32(ip: &str) -> u32 {
        let cidr = format!("{ip}/32");

        u32::from(Ipv4Net::from_str(&cidr).unwrap().addr())
    }

    struct MockRng {
        value: u32,
    }

    impl MockRng {
        fn new(value: u32) -> Self {
            Self { value }
        }
    }

    impl RandomRangeGenerator<u32> for MockRng {
        fn random_range(&mut self, _range: std::ops::Range<u32>) -> u32 {
            self.value
        }
    }

    #[test]
    fn sumarizes_an_interface() {
        let expected = "-[ipv4 : 10.1.1.1/32] - 0

[CIDR]
Host address            - 10.1.1.1
Host address (decimal)  - 167837953
Host address (hex)      - A010101
Network address         - 10.1.1.1
Network mask            - 255.255.255.255
Network mask (bits)     - 32
Network mask (hex)      - FFFFFFFF
Broadcast address       - 10.1.1.1
Cisco wildcard          - 0.0.0.0
Addresses in network    - 1
Network range           - 10.1.1.1 - 10.1.1.1
";
        let ip = Ipv4Net::from_str("10.1.1.1/32").unwrap();
        let mut ctx = create_test_ctx();

        Interface::summarize(&ip, &mut ctx).unwrap();
        let output = get_output_as_string(&ctx);

        assert_eq!(output, expected)
    }

    #[test]
    fn includes_usable_range_below_31() {
        let expected = "-[ipv4 : 10.1.1.1/30] - 0

[CIDR]
Host address            - 10.1.1.1
Host address (decimal)  - 167837953
Host address (hex)      - A010101
Network address         - 10.1.1.0
Network mask            - 255.255.255.252
Network mask (bits)     - 30
Network mask (hex)      - FFFFFFFC
Broadcast address       - 10.1.1.3
Cisco wildcard          - 0.0.0.3
Addresses in network    - 4
Network range           - 10.1.1.0 - 10.1.1.3
Usable range            - 10.1.1.1 - 10.1.1.2
";

        let ip = Ipv4Net::from_str("10.1.1.1/30").unwrap();
        let mut ctx = create_test_ctx();

        Interface::summarize(&ip, &mut ctx).unwrap();
        let output = get_output_as_string(&ctx);

        assert_eq!(output, expected)
    }

    #[test]
    fn splits_a_range() {
        let expected = "-[ipv4 : 1.2.3.4/25] - 0

[Split network]
Network - 1.2.3.0         - 1.2.3.15
Network - 1.2.3.16        - 1.2.3.31
Network - 1.2.3.32        - 1.2.3.47
Network - 1.2.3.48        - 1.2.3.63
Network - 1.2.3.64        - 1.2.3.79
Network - 1.2.3.80        - 1.2.3.95
Network - 1.2.3.96        - 1.2.3.111
Network - 1.2.3.112       - 1.2.3.127
";
        let ip = Ipv4Net::from_str("1.2.3.4/25").unwrap();
        let mut ctx = create_test_ctx();

        Interface::split(&ip, &mut ctx, 28).unwrap();
        let output = get_output_as_string(&ctx);

        assert_eq!(output, expected);
    }

    #[test]
    fn reports_oversized_range_split() {
        let ip = Ipv4Net::from_str("1.2.3.4/29").unwrap();
        let mut ctx = create_test_ctx();

        let e = Interface::split(&ip, &mut ctx, 24).unwrap_err();

        assert!(matches!(e, Error::SplitSmallerThanPrefixLen(24, 29)));
    }

    #[test]
    fn random_split_produces_different_results_with_different_random_values() {
        let ip = Ipv4Net::from_str("182.37.233.188/16").unwrap();

        let mut mock_rng1 = MockRng::new(ip_to_u32("34.86.183.34"));
        let output1 = ip.generate_random_split(24, &mut mock_rng1).unwrap();

        assert_eq!(output1.to_string(), "182.37.183.0/24");

        let mut mock_rng2 = MockRng::new(ip_to_u32("33.25.245.44"));
        let output2 = ip.generate_random_split(18, &mut mock_rng2).unwrap();

        assert_eq!(output2.to_string(), "182.37.192.0/18");

        assert_ne!(output1, output2);
    }

    #[test]
    fn random_split_works_with_netmask_0_input() {
        let ip = Ipv4Net::from_str("0.0.0.0/0").unwrap();

        let mut mock_rng = MockRng::new(ip_to_u32("230.141.13.62"));
        let output = ip.generate_random_split(24, &mut mock_rng).unwrap();

        assert_eq!(output.to_string(), "230.141.13.0/24");
    }
}
