use std::convert::TryFrom;
use std::error::Error;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use lazy_static::lazy_static;
use pecunia::prelude::*;
use pecunia::units::currency::Currency;
use stock_market_utils::derivative::{Derivative, ISIN, SYMBOL, WKN};
use stock_market_utils::order::{OrderStatus, OrderType};

use comdirect_api::api_interface::ComdirectApi;
use comdirect_api::api_types::deposit::ComdirectDeposit;
use comdirect_api::api_types::instrument::InstrumentId;
use comdirect_api::api_types::market_place::{MarketPlace, MarketPlaceFilterParameters};
use comdirect_api::api_types::order::{ComdirectOrder, OrderFilterParameters};
use comdirect_api::api_types::order::order_change::OrderChange;
use comdirect_api::api_types::order::order_outline::{OrderOutline, RawSingleOrderOutline};
use comdirect_api::api_types::position::Position;
use comdirect_api::api_types::transaction::TransactionFilterParameters;

lazy_static! {
    static ref SESSION: Arc<ComdirectApi> = Arc::new(comdirect_session().unwrap());
}

fn comdirect_session() -> Result<ComdirectApi, Box<dyn Error>> {
    let mut comdirect = new_comdirect();
    comdirect.new_session()?;
    Ok(comdirect)
}

fn new_comdirect() -> ComdirectApi {
    ComdirectApi::new(
        env!("client_id").to_string().into(),
        env!("client_secret").to_string().into(),
        env!("username").to_string().into(),
        env!("password").to_string().into(),
    )
}

fn deposit() -> ComdirectDeposit {
    SESSION
        .get_deposits()
        .unwrap()
        .swap_remove(0)
}

fn market_place() -> MarketPlace {
    let amd_isin = ISIN::try_from("US0079031078").unwrap();
    let filter_parameters = MarketPlaceFilterParameters::builder()
        .isin(&amd_isin)
        .build().unwrap();
    SESSION
        .get_marketplaces_filtered(&filter_parameters)
        .unwrap()
        .swap_remove(1)
}

fn position(deposit: &ComdirectDeposit) -> Position {
    SESSION
        .get_positions(deposit)
        .unwrap()
        .swap_remove(0)
}

fn instrument_id() -> InstrumentId {
    InstrumentId::from(Derivative::isin_from_str("US0079031078").unwrap())
}

fn order_outline<'d, 'm, 'i>(deposit: &'d ComdirectDeposit, market_place: &'m MarketPlace, instrument_id: &'i InstrumentId) -> OrderOutline<'d, 'i, 'm> {
    let raw = RawSingleOrderOutline::builder()
        .deposit(&deposit)
        .order_type(OrderType::Limit)
        .limit(Price::new(10.0, Currency::EUR))
        .market_place_id(market_place.id())
        .instrument_id(&instrument_id)
        .quantity(F64::new(1.0))
        .build()
        .unwrap();
    OrderOutline::SingleOrder(raw)
}

macro_rules! position {
    ($position:ident) => {
        let deposit = deposit();
        let $position = SESSION.get_positions(&deposit).unwrap().swap_remove(0);
    };
    (mut $position:ident) => {
        let deposit = deposit();
        let mut $position = SESSION.get_positions(&deposit).unwrap().swap_remove(0);
    };
}

macro_rules! orders {
    ($orders:ident) => {
        let deposit = deposit();
        let $orders = SESSION.get_orders(&deposit).unwrap();
    };
}

macro_rules! order_outline {
    ($order_outline:ident) => {
        let deposit = deposit();
        let market_place = market_place();
        let instrument_id = instrument_id();
        let $order_outline = order_outline(&deposit, &market_place, &instrument_id);
    };
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
    println!("\n\ndeposits: {:#?}", deposits);
}

#[test]
fn get_deposit_positions() {
    let deposit = deposit();
    let positions = SESSION.get_positions(&deposit).unwrap();
    println!("\n\npositions: {:#?}", positions);
}

#[test]
fn update_position() {
    position!(mut position);
    println!("\n\nposition: {:#?}", position);

    sleep(Duration::from_secs(2));
    SESSION.update_position(&mut position).unwrap();
    println!("position: {:#?}", position);
}

#[test]
fn get_deposit_transactions() {
    let deposit = deposit();
    let transactions = SESSION.get_deposit_transactions(&deposit).unwrap();
    println!("\n\ntransactions: {:#?}", transactions);
}

#[test]
fn get_deposit_filtered_transactions() {
    let deposit = deposit();
    let position = position(&deposit);
    let parameters = TransactionFilterParameters::default()
        .set_position_wkn(&position);

    let transactions = SESSION.get_deposit_transactions_filtered(
        &deposit,
        &parameters,
    ).unwrap();
    println!("\n\ntransactions: {:#?}", transactions);
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

    println!("\n\ninstrument by wkn: {:#?}", by_wkn);
    println!("instrument by isin: {:#?}", by_isin);
    println!("instrument by symbol: {:#?}", by_symbol);

    assert_eq!(by_wkn, by_isin);
    assert_eq!(by_wkn, by_symbol);
    assert_eq!(by_isin, by_symbol);
}

#[test]
fn get_market_places() {
    let market_places = SESSION.get_marketplaces().unwrap();
    println!("\n\nall market places: {:#?}", market_places);
}

#[test]
fn get_market_places_filtered() {
    let market_places = SESSION.get_marketplaces_filtered(&MarketPlaceFilterParameters::default()).unwrap();
    println!("\n\nmarket places filtered (default): {:#?}", market_places);
}

#[test]
fn get_orders() {
    orders!(orders);
    println!("\n\nall orders: {:#?}", orders);
}

#[test]
fn get_orders_filtered() {
    let deposit = deposit();
    let orders = SESSION.get_orders_filtered(&deposit, &OrderFilterParameters::default()).unwrap();
    println!("\n\norders filtered (default): {:#?}", orders);
}

#[test]
fn get_order() {
    let deposit = deposit();
    let orders = SESSION.get_orders(&deposit).unwrap();
    if orders.len() == 0 {
        println!("\n\nNo orders found");
        return;
    }
    let order = SESSION.get_order(&deposit, orders[0].raw().id()).unwrap();

    println!("\n\norders: {:#?}", orders);
    println!("order 0: {:#?}", order);
    assert_eq!(orders[0], order);
}



#[test]
fn pre_validate_order_outline() {
    order_outline!(order_outline);

    SESSION.pre_validate_order_outline(&order_outline).unwrap();
}

#[test]
fn order_cost_indication() {
    order_outline!(order_outline);

    SESSION.pre_validate_order_outline(&order_outline).unwrap();
    let cost_indication = SESSION.order_cost_indication(&order_outline).unwrap();
    println!("cost indication: {:#?}", cost_indication);
}

#[test]
#[ignore]
fn place_order() {
    order_outline!(order_outline);

    let order = SESSION.place_order(&order_outline).unwrap();
    println!("order: {:#?}", order);
    let _: ComdirectOrder = SESSION.get_order(&deposit(), order.id()).unwrap();
}

#[test]
#[ignore]
fn pre_validate_order_change() {
    order_outline!(order_outline);
    let mut order = SESSION.place_order(&order_outline).unwrap();

    let order_change = OrderChange::from_order0(&mut order)
        .limit(Price::new(5.0, Currency::EUR));

    SESSION.pre_validate_order_change(&order_change).unwrap();
}

#[test]
#[ignore]
fn pre_validate_order_deletion() {
    order_outline!(order_outline);
    let order = SESSION.place_order(&order_outline).unwrap();

    SESSION.pre_validate_order_deletion(&order).unwrap();
}

#[test]
fn order_change_cost_indication() {
    order_outline!(order_outline);
    let mut order = SESSION.place_order(&order_outline).unwrap();

    let order_change = OrderChange::from_order0(&mut order)
        .limit(Price::new(5.0, Currency::EUR));

    let cost_indication = SESSION.order_change_cost_indication(&order_change).unwrap();
    println!("cost indication: {:#?}", cost_indication);
}

// FIXME: see src/api_interface::ComdirectApi::order_deletion_cost_indication
// #[test]
// fn order_delete_cost_indication() {
//     order_outline!(order_outline);
//     let order = SESSION.place_order(&order_outline).unwrap();
// 
//     let cost_indication = SESSION.order_deletion_cost_indication(&order).unwrap();
//     println!("cost indication: {:#?}", cost_indication);
// }

#[test]
#[ignore]
fn change_order() {
    order_outline!(order_outline);
    let mut order = SESSION.place_order(&order_outline).unwrap();

    println!("before change: {:#?}", order);

    let order_change = OrderChange::from_order0(&mut order)
        .limit(Price::new(5.0, Currency::EUR));

    SESSION.change_order(order_change).unwrap();
    println!("after change: {:#?}", order);
}

#[test]
#[ignore]
fn delete_order() {
    use comdirect_api::error::Error;
    order_outline!(order_outline);
    let order = SESSION.place_order(&order_outline).unwrap();
    let order_id = order.id().clone();

    SESSION.delete_order(order).unwrap();

    match SESSION.get_order(&deposit(), &order_id) {
        Ok(o) => assert_eq!(o.status0(), OrderStatus::Canceled),
        Err(e) => assert_eq!(e, Error::NotFound)
    }
}
