use std::convert::TryFrom;
use std::error::Error;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use lazy_static::lazy_static;
use stock_market_utils::derivative::{Derivative, ISIN, SYMBOL, WKN};

use comdirect_api::api_interface::Comdirect;
use comdirect_api::market_place::OrderDimensionsFilterParameters;
use comdirect_api::order::OrderFilterParameters;
use comdirect_api::transaction::TransactionFilterParameters;

lazy_static! {
    static ref SESSION: Arc<Comdirect> = Arc::new(comdirect_session().unwrap());
}

fn comdirect_session() -> Result<Comdirect, Box<dyn Error>> {
    let mut comdirect = new_comdirect();
    comdirect.new_session()?;
    Ok(comdirect)
}

fn new_comdirect() -> Comdirect {
    Comdirect::new(
        env!("client_id").to_string().into(),
        env!("client_secret").to_string().into(),
        env!("username").to_string().into(),
        env!("password").to_string().into(),
    )
}

#[test]
#[ignore]
fn session() {
    //! !open Photo tan app before testing!
    //! you'll have 10 seconds to activate the push tan

    let mut comdirect = new_comdirect();

    comdirect
        .new_session()
        .unwrap();

    sleep(std::time::Duration::from_secs(10));

    comdirect
        .refresh_session()
        .unwrap();

    sleep(std::time::Duration::from_secs(10));

    comdirect
        .end_session()
        .unwrap();
}

#[test]
fn get_deposits() {
    let deposits = SESSION.get_deposits().unwrap();
    println!("\n\ndeposits: {:?}", deposits);
}

#[test]
fn get_deposit_positions() {
    let deposits = SESSION.get_deposits().unwrap();
    let positions = SESSION.get_positions(&deposits[0]).unwrap();
    println!("\n\npositions: {:?}", positions);
}

#[test]
fn update_position() {
    let deposits = SESSION.get_deposits().unwrap();
    let mut positions = SESSION.get_positions(&deposits[0]).unwrap();
    println!("\n\nposition: {:?}", positions[0]);

    sleep(Duration::from_secs(5));
    SESSION.update_position(&mut positions[0]).unwrap();
    println!("position: {:?}", positions[0]);
}

#[test]
fn get_deposit_transactions() {
    let deposits = SESSION.get_deposits().unwrap();
    let transactions = SESSION.get_deposit_transactions(
        &deposits[0],
        &TransactionFilterParameters::default(),
    ).unwrap();
    println!("\n\ntransactions: {:?}", transactions);
}

#[test]
fn get_deposit_filtered_transactions() {
    let deposits = SESSION.get_deposits().unwrap();
    let positions = SESSION.get_positions(&deposits[0]).unwrap();

    let parameters = TransactionFilterParameters::default()
        .set_position_wkn(&positions[0]);

    let transactions = SESSION.get_deposit_transactions(
        &deposits[0],
        &parameters,
    ).unwrap();
    println!("\n\ntransactions: {:?}", transactions);
}

#[test]
fn get_instrument() {

    // McDonald's
    let wkn = Derivative::WKN(WKN::try_from("856958").unwrap());
    let isin = Derivative::ISIN(ISIN::try_from("US5801351017").unwrap());
    let symbol = Derivative::SYMBOL(SYMBOL::try_from("MDO").unwrap());

    let by_wkn = SESSION.get_instrument(&wkn).unwrap();
    let by_isin = SESSION.get_instrument(&isin).unwrap();
    let by_symbol = SESSION.get_instrument(&symbol).unwrap();

    println!("\n\ninstrument by wkn: {:?}", by_wkn);
    println!("instrument by isin: {:?}", by_isin);
    println!("instrument by symbol: {:?}", by_symbol);

    assert_eq!(by_wkn, by_isin);
    assert_eq!(by_wkn, by_symbol);
    assert_eq!(by_isin, by_symbol);
}

#[test]
fn get_market_places() {
    let without_parameters = SESSION.get_marketplaces(&OrderDimensionsFilterParameters::default()).unwrap();
    println!("\n\norder dimensions without parameters: {:?}", without_parameters);
}

#[test]
fn get_orders() {
    let deposits = SESSION.get_deposits().unwrap();
    let without_parameters = SESSION.get_orders(&deposits[0], &OrderFilterParameters::default()).unwrap();
    println!("\n\norders without parameters: {:?}", without_parameters);
}
