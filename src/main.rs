mod subnet;
mod ipv4_subnet;

use clap::Parser;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use std::net::{IpAddr, Ipv4Addr};
use std::process;
use std::str::FromStr;
use crate::subnet::Subnet;

#[derive(Parser)]
struct Cli {
    ip: String,
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
            let subnet = ipv4_subnet::Ipv4Subnet::new(ipv4);
            subnet.print();
        }
        IpNet::V6(ipv6) => {
            println!("ip is: {}", ipv6.addr());
        }
    }
}
