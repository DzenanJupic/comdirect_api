use pecunia::prelude::*;
use pecunia::units::currency::Currency;
use serde::Deserialize;

use crate::types::order::ComdirectOrder;
use crate::types::order::order_change::OrderChange;
use crate::types::order::order_outline::OrderOutline;

#[derive(Debug, PartialEq, getset::Getters)]
#[getset(get = "pub")]
pub struct CostIndication<'o, 'd, 'i, 'm> {
    order_outline: &'o OrderOutline<'d, 'i, 'm>,
    raw: RawCostIndication,
}

#[derive(Debug, PartialEq)]
pub enum ChangeCostIndication<'oc, 'o, 'd> {
    Change {
        order_change: &'oc OrderChange<'o>,
        raw: RawCostIndication,
    },
    Delete {
        order: &'o ComdirectOrder<'d>,
        raw: RawCostIndication,
    },
}


#[derive(Clone, Debug, Deserialize, PartialEq, getset::Getters)]
#[getset(get = "pub")]
#[serde(rename_all = "camelCase")]
pub struct RawCostIndication {
    #[serde(default)]
    calculation_successful: bool,
    #[serde(with = "crate::serde::amount_value::price")]
    expected_value: Price,
    settlement_currency: Currency,
    trading_currency: Currency,
    reporting_currency: Currency,
    #[serde(rename = "fxRate")]
    exchange_rate: Option<ExchangeRate>,
    #[serde(with = "crate::serde::amount_value::price::option")]
    expected_settlement_costs: Option<Price>,
    purchase_costs: Option<CostGroup>,
    holding_costs: Option<CostGroup>,
    sales_costs: CostGroup,
    holding_period: Option<F64>,
    #[serde(rename = "totalCostsAbs")]
    #[serde(with = "crate::serde::amount_value::price")]
    total_costs: Price,
    #[serde(rename = "totalCostsRel")]
    total_costs_relative: Percent,
    #[serde(rename = "totalCostsDetail")]
    cost_detail: TotalCostDetail,
    total_holding_costs: TotalHoldingCosts,
}

#[derive(Clone, Debug, Deserialize, PartialEq, getset::Getters)]
pub struct ExchangeRate {
    #[serde(with = "crate::serde::amount_value::price")]
    bid: Price,
    #[serde(with = "crate::serde::amount_value::price")]
    ask: Price,
}

#[derive(Clone, Debug, Deserialize, PartialEq, getset::Getters)]
#[serde(rename_all = "camelCase")]
pub struct CostGroup {
    #[serde(rename = "type")]
    group_type: CostGroupType,
    #[serde(rename = "sum")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    sum_trading_currency: Option<Price>,
    #[serde(with = "crate::serde::amount_value::price")]
    sum_reporting_currency: Price,
    #[serde(default)]
    costs: Vec<CostEntry>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum CostGroupType {
    #[serde(rename = "K")]
    PurchaseCosts,
    #[serde(rename = "H")]
    HoldingCosts,
    #[serde(rename = "V")]
    SellingCosts,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CostEntry {
    #[serde(rename = "type")]
    entry_type: CostEntryType,
    #[serde(rename = "amount")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    amount_trading_currency: Option<Price>,
    #[serde(with = "crate::serde::amount_value::price")]
    amount_reporting_currency: Price,
    inducement: Option<Inducement>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum CostEntryType {
    #[serde(rename = "E")]
    InternCosts,
    #[serde(rename = "F")]
    ExternCosts,
    #[serde(rename = "P")]
    ProductionCosts,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct Inducement {
    #[serde(with = "crate::serde::amount_value::price")]
    amount: Price,
    #[serde(default)]
    estimated: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TotalCostDetail {
    service_costs: TotalCostEntry,
    #[serde(with = "crate::serde::amount_value::price")]
    service_inducement: Price,
    external_costs: TotalCostEntry,
    product_costs: TotalCostEntry,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TotalCostEntry {
    #[serde(rename = "type")]
    entry_type: CostEntryType,
    #[serde(with = "crate::serde::amount_value::price")]
    amount: Price,
    #[serde(rename = "averageReturnPA")]
    average_return_pa: Option<Percent>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct TotalHoldingCosts {
    year1: TotalHoldingCostEntry,
    year2: TotalHoldingCostEntry,
    sales: TotalHoldingCostEntry,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TotalHoldingCostEntry {
    #[serde(rename = "type")]
    entry_type: TotalHoldingCostEntryType,
    #[serde(with = "crate::serde::amount_value::price")]
    amount: Price,
    #[serde(rename = "averageReturnPA")]
    average_return_pa: Option<Percent>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum TotalHoldingCostEntryType {
    #[serde(rename = "IM_ERSTEN_JAHR")]
    FirstYear,
    #[serde(rename = "IM_ZWEITEN_JAHR")]
    SecondYear,
    #[serde(rename = "IM_JAHR_DER_VERAUESSERUNG")]
    YearOfSell,
}

impl<'o, 'd, 'i, 'm> CostIndication<'o, 'd, 'i, 'm> {
    #[inline(always)]
    pub fn into_raw(self) -> RawCostIndication {
        self.raw
    }

    #[inline(always)]
    pub(crate) const fn from_raw(raw: RawCostIndication, order_outline: &'o OrderOutline<'d, 'i, 'm>) -> Self {
        Self { order_outline, raw }
    }
}

impl ChangeCostIndication<'_, '_, '_> {
    #[inline(always)]
    pub fn into_raw(self) -> RawCostIndication {
        use ChangeCostIndication::*;
        match self {
            Change { raw, .. } => raw,
            Delete { raw, .. } => raw
        }
    }
}
