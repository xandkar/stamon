use anyhow::Result;
use clap::Parser;

fn helium_fetch_balance_hnt(account: &str) -> Result<f64> {
    let url = format!("https://api.helium.io/v1/accounts/{}", account);
    let body = reqwest::blocking::get(url)?;
    let payload = body.text()?;
    let parsed: serde_json::Value = serde_json::from_str(&payload)?;
    match &parsed["data"]["balance"] {
        serde_json::Value::Number(balance) => {
            let balance = balance.as_f64().unwrap() / 100000000.0;
            log::debug!("HNT account balance: {:?}", balance);
            Ok(balance)
        }
        balance => {
            let msg = format!("unexpected balance format: {:?}", balance);
            Err(anyhow::Error::msg(msg))
        }
    }
}

fn binance_fetch_average_price(symbol: &str) -> Result<f64> {
    let market: binance::market::Market = binance::api::Binance::new(None, None);
    match market.get_average_price(symbol) {
        Err(e) => {
            // TODO How to propagate the error?
            //       "`(dyn std::error::Error + Send + 'static)`
            //       cannot be shared between threads safely"
            let msg = format!("binance API failure: {:?}", e);
            Err(anyhow::Error::msg(msg))
        }
        Ok(average_price) => {
            let average_price = average_price.price;
            log::debug!("average_price {}: {}", symbol, average_price);
            Ok(average_price)
        }
    }
}

#[derive(Debug, Parser)]
struct Cli {
    account: String,

    #[clap(default_value_t = 60)]
    interval: u64,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Cli::parse();
    // TODO Independent intervals of update, but recompute when either updated,
    //      with the other pulled from cache.
    log::info!("starting with args: {:?}", &args);
    loop {
        let price_hnt_in_usdt = binance_fetch_average_price("HNTUSDT")?;
        let balance_hnt = helium_fetch_balance_hnt(&args.account)?;
        let balance_usdt = price_hnt_in_usdt * balance_hnt;
        log::info!(
            "data update success. \
            price_hnt_in_usdt:{}, balance_hnt:{}, balance_usdt:{}",
            price_hnt_in_usdt,
            balance_hnt,
            balance_usdt
        );
        println!(
            "H {:.2} ${:.2} ${:.2}",
            balance_hnt, price_hnt_in_usdt, balance_usdt
        );
        std::thread::sleep(std::time::Duration::from_secs(args.interval));
    }
}
