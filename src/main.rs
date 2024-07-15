use clap::Parser;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use std::net::{IpAddr, Ipv4Addr};
use std::process;
use std::str::FromStr;

#[derive(Parser)]
struct Cli {
    ip: String,
}

fn ipv4_addr_to_u32(ip: Ipv4Addr) -> u32 {
    u32::from_be_bytes(ip.octets())
}

fn u32_to_ipv4_addr(ip: u32) -> Ipv4Addr {
    Ipv4Addr::from(ip.to_be_bytes())
}

fn parse_ip(ip: &str) -> Result<IpNet, Box<dyn std::error::Error>> {
    let parsed_ip = match IpAddr::from_str(ip) {
        Ok(IpAddr::V4(ipv4)) => IpNet::V4(Ipv4Net::new(ipv4, 32)?),
        Ok(IpAddr::V6(ipv6)) => IpNet::V6(Ipv6Net::new(ipv6, 128)?),
        Err(e) => {
            if let Ok(ipv4) = Ipv4Net::from_str(ip) {
                IpNet::V4(ipv4)
            } else if let Ok(ipv6) = Ipv6Net::from_str(ip) {
                IpNet::V6(ipv6)
            } else {
                return Err(Box::new(e))
            }
        }
    };

    Ok(parsed_ip)
}

fn main() {
    let args = Cli::parse();
    let ip_input = &args.ip;

    let ip = match parse_ip(ip_input) {
        Ok(ip) => { ip },
        Err(_) => {
            eprintln!("Invalid IP address");
            process::exit(1);
        }
    };

    match ip {
        IpNet::V4(ipv4) => {
            println!("-[ipv4 : {}] - 0", ipv4);
            println!("[CIDR]");
            let host_address_decimal = ipv4_addr_to_u32(ipv4.addr());
            println!("{0: <25}- {1}", "Host address", ipv4.addr());
            println!("{0: <25}- {1}", "Host address (decimal)", host_address_decimal);
            println!("{0: <25}- {1:X}", "Host address (hex)", host_address_decimal);
            println!("{0: <25}- {1}", "Network address", ipv4.network());
            println!("{0: <25}- {1}", "Network mask", ipv4.netmask());
            println!("{0: <25}- {1}", "Network mask (bits)", ipv4.prefix_len());
            let netmask_decimal = ipv4_addr_to_u32(ipv4.netmask());
            println!("{0: <25}- {1:X}", "Network mask (hex)", netmask_decimal);
            println!("{0: <25}- {1}", "Broadcast address", ipv4.broadcast());
            let cisco_wildcard = u32_to_ipv4_addr(!netmask_decimal);
            println!("{0: <25}- {1}", "Cisco wildcard", cisco_wildcard);
            let addresses_in_network = 2u32.pow(32 - ipv4.prefix_len() as u32);
            println!(
                "{0: <25}- {1}",
                "Addresses in network", addresses_in_network
            );
            let network_range_start = ipv4.network();
            let network_range_end = ipv4.broadcast();
            println!(
                "{0: <25}- {1}",
                "Network range",
                format!("{} - {}", network_range_start, network_range_end)
            );

            let usable_range_start = if ipv4.prefix_len() < 31 {
                u32_to_ipv4_addr(ipv4_addr_to_u32(network_range_start) + 1)
            } else {
                network_range_start
            };
            let usable_range_end = if ipv4.prefix_len() < 31 {
                u32_to_ipv4_addr(ipv4_addr_to_u32(network_range_end) - 1)
            } else {
                network_range_end
            };
            println!(
                "{0: <25}- {1}",
                "Usable range",
                format!("{} - {}", usable_range_start, usable_range_end)
            );
        }
        IpNet::V6(ipv6) => {
            println!("ip is: {}", ipv6.addr());
        }
    }
}
