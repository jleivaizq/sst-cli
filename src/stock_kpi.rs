use chrono::{NaiveDate,};
use chrono::prelude::*;
use yahoo_finance_api as yahoo;
use yahoo::{Quote, YahooError};
use log::{info, error};

#[derive(Debug)]
pub struct StocKpi {
    pub from: NaiveDate,
    pub symbol: String,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub price_diff: Option<(f64, f64)>,

    sma: Option<Vec<f64>>,
    quotes: Option<Vec<Quote>>,
}

impl StocKpi {
    pub fn new(symbol: &str, from_src: NaiveDate) -> Self {
    StocKpi {
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

    pub fn last_quote(self: &StocKpi) -> Option<&Quote> {
        if let Some(quotes) = &self.quotes{
            Some(quotes.last().unwrap())
        } else {
            None
        }
    }

    pub fn last_sma(self: &StocKpi) -> Option<f64> {
        if let Some(sma) = &self.sma {
            Some(sma.last().unwrap().clone())
        } else {
            None
        }
    }

    pub fn calculate(self: &mut StocKpi, sma_window: usize) { 
        match StocKpi::retrieve_quotes(&self.symbol, &self.from) {
            Ok(quotes) => {
                self.min = StocKpi::min(&quotes[..]);
                self.max = StocKpi::max(&quotes[..]);
                let adjclose_series : Vec<f64> = quotes.iter().map(|q|q.adjclose).collect();
                self.sma = StocKpi::n_window_sma(sma_window, &adjclose_series[..]);
                self.price_diff = StocKpi::price_diff(&adjclose_series[..]);
                self.quotes = Some(quotes);
            },
            Err(error) => {
                error!("Could not calculate performance metrics for {} from {}. Error: {}", 
                    self.symbol, self.from, error);
            }
        }
    }

}