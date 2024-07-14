use clap::Parser;


#[derive(Parser)]
struct Cli {
    ip: String,
}

fn main() {
    let args = Cli::parse();

    println!("Value for ip: {}", args.ip);
}
