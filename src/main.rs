use clap::Parser;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use std::str::FromStr;
use std::process;

#[derive(Parser)]
struct Cli {
    ip: String,
}

fn main() {
    let args = Cli::parse();
    let ip_input = &args.ip;

    let ip = if let Ok(ipv4) = Ipv4Net::from_str(ip_input) {
        IpNet::V4(ipv4)
    } else if let Ok(ipv6) = Ipv6Net::from_str(ip_input) {
        IpNet::V6(ipv6)
    } else {
        eprintln!("Invalid IP address");
        process::exit(1);
    };

    println!("ip is: {}", ip);
}
