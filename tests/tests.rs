use std::convert::TryFrom;
use std::error::Error;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use pecunia::iso_codes::units::NotAUnit;
use pecunia::market_values::f64::F64;
use pecunia::market_values::unit_value::UnitValue;
use lazy_static::lazy_static;
use stock_market_utils::derivative::{Derivative, ISIN, SYMBOL, WKN};

use comdirect_api::api_interface::Comdirect;
use comdirect_api::instrument::InstrumentId;
use comdirect_api::market_place::OrderDimensionsFilterParameters;
use comdirect_api::order::OrderFilterParameters;
use comdirect_api::order_outline::{ComdirectOrderOutline, RawSingleOrderOutline};
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
#[ignore]
fn get_deposits() {
    let deposits = SESSION.get_deposits().unwrap();
    println!("\n\ndeposits: {:#?}", deposits);
}

#[test]
#[ignore]
fn get_deposit_positions() {
    let deposits = SESSION.get_deposits().unwrap();
    let positions = SESSION.get_positions(&deposits[0]).unwrap();
    println!("\n\npositions: {:#?}", positions);
}

#[test]
#[ignore]
fn update_position() {
    let deposits = SESSION.get_deposits().unwrap();
    let mut positions = SESSION.get_positions(&deposits[0]).unwrap();
    println!("\n\nposition: {:#?}", positions[0]);

    sleep(Duration::from_secs(2));
    SESSION.update_position(&mut positions[0]).unwrap();
    println!("position: {:#?}", positions[0]);
}

#[test]
#[ignore]
fn get_deposit_transactions() {
    let deposits = SESSION.get_deposits().unwrap();
    let transactions = SESSION.get_deposit_transactions(
        &deposits[0],
        &TransactionFilterParameters::default(),
    ).unwrap();
    println!("\n\ntransactions: {:#?}", transactions);
}

#[test]
#[ignore]
fn get_deposit_filtered_transactions() {
    let deposits = SESSION.get_deposits().unwrap();
    let positions = SESSION.get_positions(&deposits[0]).unwrap();

    let parameters = TransactionFilterParameters::default()
        .set_position_wkn(&positions[0]);

    let transactions = SESSION.get_deposit_transactions(
        &deposits[0],
        &parameters,
    ).unwrap();
    println!("\n\ntransactions: {:#?}", transactions);
}

#[test]
#[ignore]
fn get_instrument() {

    // McDonald's
    let wkn = Derivative::WKN(WKN::try_from("856958").unwrap());
    let isin = Derivative::ISIN(ISIN::try_from("US5801351017").unwrap());
    let symbol = Derivative::SYMBOL(SYMBOL::try_from("MDO").unwrap());

    let by_wkn = SESSION.get_instrument(&wkn).unwrap();
    let by_isin = SESSION.get_instrument(&isin).unwrap();
    let by_symbol = SESSION.get_instrument(&symbol).unwrap();

    println!("\n\ninstrument by wkn: {:#?}", by_wkn);
    println!("instrument by isin: {:#?}", by_isin);
    println!("instrument by symbol: {:#?}", by_symbol);

    assert_eq!(by_wkn, by_isin);
    assert_eq!(by_wkn, by_symbol);
    assert_eq!(by_isin, by_symbol);
}

#[test]
#[ignore]
fn get_market_places() {
    let without_parameters = SESSION.get_marketplaces(&OrderDimensionsFilterParameters::default()).unwrap();
    println!("\n\nmarket places without parameters: {:#?}", without_parameters);
}

#[test]
#[ignore]
fn get_orders() {
    let deposits = SESSION.get_deposits().unwrap();
    let without_parameters = SESSION.get_orders(&deposits[0], &OrderFilterParameters::default()).unwrap();
    println!("\n\norders without parameters: {:#?}", without_parameters);
}

#[test]
#[ignore]
fn get_order() {
    let deposits = SESSION.get_deposits().unwrap();

    let orders = SESSION.get_orders(&deposits[0], &OrderFilterParameters::default()).unwrap();
    if orders.len() == 0 {
        println!("\n\nNo orders found");
        return;
    }
    let order = SESSION.get_order(&deposits[0], orders[0].raw().id()).unwrap();

    println!("\n\norders: {:#?}", orders);
    println!("order 0: {:#?}", order);
    assert_eq!(orders[0], order);
}

#[test]
fn pre_validate_default_order_outline() {
    // let order_outline = ComdirectOrderOutline::build();
    // SESSION.pre_validate_order(&order_outline).unwrap();
    unimplemented!()
}

#[test]
fn pre_validate_order_outline() {
    let deposits = SESSION.get_deposits().unwrap();
    let market_places = SESSION.get_marketplaces(&OrderDimensionsFilterParameters::default()).unwrap();
    let mc_donald_s = InstrumentId::from(Derivative::ISIN(ISIN::try_from("US5801351017").unwrap()));

    let order_outline = RawSingleOrderOutline::builder()
        .deposit_id(deposits[0].id())
        .market_place_id(market_places[0].id())
        .instrument_id(&mc_donald_s)
        .quantity(UnitValue::new(F64::new(1.0), NotAUnit))
        .build()
        .unwrap();

    let order_outline = ComdirectOrderOutline::SingleOrder(order_outline);
    SESSION.pre_validate_order(&order_outline).unwrap();
}
