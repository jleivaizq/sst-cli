use structopt::StructOpt;
use log::{info};

#[derive(Debug)]
#[derive(StructOpt)]
struct Cli {
    #[structopt(short = "s", long = "symbols")]
    symbols: Vec<String>,
    #[structopt(short = "f", long = "from")]
    from: String,
}


fn main() {
    env_logger::init();
    let args = Cli::from_args();
    info!("starting up");
    println!("{:?}", args);
}