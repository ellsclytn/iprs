use std::io::Write;
use std::{fmt, net::Ipv4Addr};

use crate::context::Ctx;
use crate::error::{Error, Result};
use crate::interface::Interface;
use ipnet::Ipv4Net;

pub trait RandomRangeGenerator {
    fn random_range(&mut self, range: std::ops::Range<u32>) -> u32;
}

pub struct DefaultRng;

impl RandomRangeGenerator for DefaultRng {
    fn random_range(&mut self, range: std::ops::Range<u32>) -> u32 {
        rand::random_range(range)
    }
}

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
    format!("{name: <24}- {value}")
}

impl Interface for Ipv4Net {
    fn summarize<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>) -> Result<()> {
        ctx.writeln(format!("-[ipv4 : {self}] - 0\n\n[CIDR]"))?;

        ctx.writeln(format_attribute("Host address", self.addr()))?;
        ctx.writeln(format_attribute(
            "Host address (decimal)",
            self.addr().to_u32(),
        ))?;
        ctx.writeln(format_attribute(
            "Host address (hex)",
            format!("{:X}", self.addr().to_u32()),
        ))?;
        ctx.writeln(format_attribute("Network address", self.network()))?;
        ctx.writeln(format_attribute("Network mask", self.netmask()))?;
        ctx.writeln(format_attribute("Network mask (bits)", self.prefix_len()))?;
        ctx.writeln(format_attribute(
            "Network mask (hex)",
            format!("{:X}", self.netmask().to_u32()),
        ))?;
        ctx.writeln(format_attribute("Broadcast address", self.broadcast()))?;
        ctx.writeln(format_attribute(
            "Cisco wildcard",
            (!self.netmask().to_u32()).to_ipv4(),
        ))?;
        ctx.writeln(format_attribute(
            "Addresses in network",
            self.addresses_in_network(),
        ))?;
        ctx.writeln(format_attribute("Network range", self.network_range()))?;

        if let Some(usable_range) = self.usable_range() {
            ctx.writeln(format_attribute("Usable range", usable_range))?;
        }

        Ok(())
    }

    fn split<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>, mask: u8) -> Result<()> {
        ctx.writeln(format!("-[ipv4 : {self}] - 0\n"))?;
        ctx.writeln("[Split network]".to_string())?;

        match self.subnets(mask) {
            Ok(subnets) => {
                for subnet in subnets {
                    ctx.writeln(format!(
                        "Network - {:<15} - {}",
                        subnet.addr(),
                        subnet.broadcast()
                    ))?;
                }

                Ok(())
            }
            Err(_) => Err(Error::SplitSmallerThanPrefixLen(mask, self.prefix_len())),
        }
    }

    fn random_split<W: Write, E: Write>(&self, ctx: &mut Ctx<W, E>, split: u8) -> Result<()> {
        let mut rng = DefaultRng;
        random_split_with_rng(self, ctx, split, &mut rng)
    }
}

fn random_split_with_rng<W: Write, E: Write, R: RandomRangeGenerator>(
    ip: &Ipv4Net,
    ctx: &mut Ctx<W, E>,
    split: u8,
    rng: &mut R,
) -> Result<()> {
    let address = random_split_generate_address(ip, split, rng)?;
    address.summarize(ctx)?;

    Ok(())
}

fn random_split_generate_address<R: RandomRangeGenerator>(
    ip: &Ipv4Net,
    split: u8,
    rng: &mut R,
) -> Result<Ipv4Net> {
    if split <= ip.prefix_len() {
        return Err(Error::SplitSmallerThanPrefixLen(split, ip.prefix_len()));
    } else if split > 32 {
        return Err(Error::SplitTooBig(32, split));
    }

    let random_number = rng.random_range(0..u32::MAX);
    let split_mask = !((1 << (32 - split)) - 1);

    if ip.prefix_len() == 0 {
        let random_address = random_number & split_mask;
        return Ok(Ipv4Net::new(random_address.to_ipv4(), split)?);
    }

    let supernet_mask = (1 << (32 - ip.prefix_len())) - 1;
    let new_address: u32 = (ip.addr().to_u32() & !supernet_mask) | (random_number & supernet_mask);
    let new_prefix = new_address & split_mask;
    let address = Ipv4Net::new(new_prefix.to_ipv4(), split)?;

    Ok(address)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::context::test_util::{create_test_ctx, get_output_as_string};

    use pretty_assertions::assert_eq;

    fn ip_to_u32(ip: &str) -> u32 {
        let cidr = format!("{ip}/32");

        Ipv4Net::from_str(&cidr).unwrap().addr().to_u32()
    }

    struct MockRng {
        value: u32,
    }

    impl MockRng {
        fn new(value: u32) -> Self {
            Self { value }
        }
    }

    impl RandomRangeGenerator for MockRng {
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

        ip.summarize(&mut ctx).unwrap();
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

        ip.summarize(&mut ctx).unwrap();
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

        ip.split(&mut ctx, 28).unwrap();
        let output = get_output_as_string(&ctx);

        assert_eq!(output, expected);
    }

    #[test]
    fn reports_oversized_range_split() {
        let ip = Ipv4Net::from_str("1.2.3.4/29").unwrap();
        let mut ctx = create_test_ctx();

        let e = ip.split(&mut ctx, 24).unwrap_err();

        assert!(matches!(e, Error::SplitSmallerThanPrefixLen(24, 29)));
    }

    #[test]
    fn random_split_produces_different_results_with_different_random_values() {
        let ip = Ipv4Net::from_str("182.37.233.188/16").unwrap();

        let mut mock_rng1 = MockRng::new(ip_to_u32("34.86.183.34"));
        let output1 = random_split_generate_address(&ip, 24, &mut mock_rng1).unwrap();

        assert_eq!(output1.to_string(), "182.37.183.0/24");

        let mut mock_rng2 = MockRng::new(ip_to_u32("33.25.245.44"));
        let output2 = random_split_generate_address(&ip, 18, &mut mock_rng2).unwrap();

        assert_eq!(output2.to_string(), "182.37.192.0/18");

        assert_ne!(output1, output2);
    }

    #[test]
    fn random_split_works_with_netmask_0_input() {
        let ip = Ipv4Net::from_str("0.0.0.0/0").unwrap();

        let mut mock_rng = MockRng::new(ip_to_u32("230.141.13.62"));
        let output = random_split_generate_address(&ip, 24, &mut mock_rng).unwrap();

        assert_eq!(output.to_string(), "230.141.13.0/24");
    }
}
