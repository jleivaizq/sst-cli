mod stock_kpi;

use structopt::StructOpt;
use chrono::{NaiveDate,};
use chrono::format::ParseError;
use chrono::prelude::*;
use log::{info};
use stock_kpi::StocKpi;

#[derive(Debug)]
#[derive(StructOpt)]
struct Cli {

    #[structopt(short = "s", long = "symbols", help = "Security symbols to gather metrics from")]
    symbols: Vec<String>,

    #[structopt(short = "f", long = "from", help = "Date from which metrics will be calculated (YYYY-MM-DD)", 
                parse(try_from_str=parse_date))]
    from: NaiveDate,

    #[structopt(short = "n", long = "window", default_value = "30",
                help = "Days to be used for the simpple moving average")]
    window: usize
}

fn parse_date(src: &str) -> Result<NaiveDate, ParseError> {
    let date_fmt = "%Y-%m-%d";
    NaiveDate::parse_from_str(&src, &date_fmt)
}

fn main() {
    env_logger::init();
    let args = Cli::from_args();
    info!("Starting up with the following args: {:?}", args);

    if !args.symbols.is_empty() {
        println!("period start,symbol,price,change %,min,max,{}d avg", args.window);
        for symbol in args.symbols {
            let mut stock = StocKpi::new(&symbol, args.from);
            stock.calculate(args.window);
            if let Some(last_quote) = stock.last_quote() {
                let price_diff = stock.price_diff;
                println!("{timestamp},{symbol},${price:.2},{change:.2}%,${min:.2},${max:.2},{last_sma:.2}", 
                        timestamp=NaiveDateTime::from_timestamp(last_quote.timestamp as i64, 0).format("%Y-%m-%dT%H:%M:%S"),
                        symbol=stock.symbol, 
                        price=last_quote.close, 
                        change=price_diff.unwrap().1, 
                        min=stock.min.unwrap(), 
                        max=stock.max.unwrap(), 
                        last_sma=stock.last_sma().unwrap());
            }
        }
    }
}