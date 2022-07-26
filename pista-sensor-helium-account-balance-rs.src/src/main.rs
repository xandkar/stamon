use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    account: String,

    #[clap(default_value_t = 60)]
    interval: u64,
}

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
    let market: binance::market::Market =
        binance::api::Binance::new(None, None);
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

fn main_loop(account: &str, interval: u64) {
    // TODO Independent intervals of update, but recompute when either updated,
    //      with the other pulled from cache.
    // TODO Maybe add max age/TTL?
    let mut price_hnt_in_usdt: Option<f64> = None;
    let mut balance_hnt: Option<f64> = None;

    loop {
        match helium_fetch_balance_hnt(account) {
            Err(e) => log::error!("helium data fetch failure: {:?}", e),
            Ok(balance) => {
                log::info!(
                    "helium data fetch success. balance_hnt:{}.",
                    balance
                );
                balance_hnt = Some(balance);
            }
        };
        match binance_fetch_average_price("HNTUSDT") {
            Err(e) => log::error!("binance data fetch failure: {:?}", e),
            Ok(price) => {
                log::info!(
                    "binance data fetch success. price_hnt_in_usdt:{}.",
                    price
                );
                price_hnt_in_usdt = Some(price);
            }
        };
        match (price_hnt_in_usdt, balance_hnt) {
            (None, None) => {
                log::debug!("neither data is yet available.");
                println!("H __.__ $__.__ $__.__");
            }
            (None, Some(balance_hnt)) => {
                println!("H {:.2} $__:__ $__:__", balance_hnt)
            }
            (Some(price_hnt_in_usdt), None) => {
                println!("H __:__ ${:.2} $__:__", price_hnt_in_usdt)
            }
            (Some(price_hnt_in_usdt), Some(balance_hnt)) => {
                let balance_usdt = price_hnt_in_usdt * balance_hnt;
                println!(
                    "H {:.2} ${:.2} ${:.2}",
                    balance_hnt, price_hnt_in_usdt, balance_usdt
                )
            }
        };
        std::thread::sleep(std::time::Duration::from_secs(interval));
    }
}

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("debug"),
    )
    .init();
    let args = Cli::parse();
    log::info!("starting with args: {:?}", &args);
    main_loop(&args.account, args.interval);
}
