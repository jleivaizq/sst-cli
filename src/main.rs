use structopt::StructOpt;
use chrono::{NaiveDate,};
use chrono::format::ParseError;
use chrono::prelude::*;
use yahoo_finance_api as yahoo;
use yahoo::{Quote, YahooError};
use log::{info, error};

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

#[derive(Debug)]
struct StockPi {
    from: NaiveDate,
    symbol: String,
    min: Option<f64>,
    max: Option<f64>,
    sma: Option<Vec<f64>>,
    price_diff: Option<(f64, f64)>,
    quotes: Option<Vec<Quote>>,
}

impl StockPi {
    pub fn new(symbol: &str, from_src: NaiveDate) -> Self {
       StockPi {
           from: from_src,
           symbol: String::from(symbol),
           min: None,
           max: None,
           sma: None,
           price_diff: None,
           quotes: None,
       } 
    }

    fn retrieve_quotes(symbol: &str, from: &NaiveDate) -> Result<Vec<Quote>, YahooError> {
        info!("Downloading values for symbol {} from {}", symbol, from);
        let provider = yahoo::YahooConnector::new();
        let start = Utc.ymd(from.year(), from.month(), from.day()).and_hms_milli(0, 0, 0, 0);
        let end = Utc::now();
        let result = provider.get_quote_history(&symbol, start, end)?;
        result.quotes()
    }

    fn min(series: &[Quote]) -> Option<f64> {
        series.iter()
              .map(|quote| quote.low)
              .fold(None, |min, low| match min {
                  None => Some(low),
                  Some(value) => if value < low { Some(value) } else { Some(low) }
              })
    }

    fn max(series: &[Quote]) -> Option<f64> {
        series.iter()
              .map(|quote| quote.high)
              .fold(None, |max, high| match max {
                  None => Some(high),
                  Some(value) => if value > high { Some(value) } else { Some(high) }
              })
    }

    fn n_window_sma(n: usize, series: &[f64]) -> Option<Vec<f64>> {
        if series.is_empty() {
            None
        } else {
            Some(series.chunks_exact(n).map(|c| c.iter().sum::<f64>() / n as f64).collect())
        }
    }

    fn price_diff(series: &[f64]) -> Option<(f64, f64)> {
        if series.is_empty() {
            None
        } else {
            let first_price = series.first().unwrap();
            let last_price = series.last().unwrap();
            let price_diff = (first_price - last_price).abs();
            Some((price_diff / first_price, price_diff))
        }
    }

    pub fn last_quote(self: &StockPi) -> Option<&Quote> {
        if let Some(quotes) = &self.quotes{
            Some(quotes.last().unwrap())
        } else {
            None
        }
    }

    pub fn last_sma(self: &StockPi) -> Option<f64> {
        if let Some(sma) = &self.sma {
            Some(sma.last().unwrap().clone())
        } else {
            None
        }
    }

    pub fn calculate(self: &mut StockPi, sma_window: usize) { 
        match StockPi::retrieve_quotes(&self.symbol, &self.from) {
            Ok(quotes) => {
                self.min = StockPi::min(&quotes[..]);
                self.max = StockPi::max(&quotes[..]);
                let adjclose_series : Vec<f64> = quotes.iter().map(|q|q.adjclose).collect();
                self.sma = StockPi::n_window_sma(sma_window, &adjclose_series[..]);
                self.price_diff = StockPi::price_diff(&adjclose_series[..]);
                self.quotes = Some(quotes);
            },
            Err(error) => {
                error!("Could not calculate performance metrics for {} from {}. Error: {}", 
                       self.symbol, self.from, error);
            }
        }
    }

}

fn main() {
    env_logger::init();
    let args = Cli::from_args();
    info!("Starting up with the following args: {:?}", args);

    if !args.symbols.is_empty() {
        println!("period start,symbol,price,change %,min,max,{}d avg", args.window);
        for symbol in args.symbols {
            let mut stock = StockPi::new(&symbol, args.from);
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