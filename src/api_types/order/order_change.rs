use pecunia::prelude::*;
use serde::{Serialize, Serializer};
use stock_market_utils::order::OrderValidity;

use crate::api_types::order::{ComdirectOrder, OrderId, RawOrder, RawSingleOrder};

pub(crate) enum OrderChangeValidation<'o, 'd, 'oc> {
    Change(&'oc OrderChange<'o>),
    Delete(&'o ComdirectOrder<'d>),
}

pub(crate) enum OrderChangeAction<'o, 'd> {
    Change(OrderChange<'o>),
    Delete(ComdirectOrder<'d>),
}

#[derive(Debug, Serialize, PartialEq, getset::Getters)]
#[getset(get = "pub with_prefix")]
#[serde(rename_all = "camelCase")]
pub struct OrderChange<'o> {
    #[serde(rename = "orderId")]
    #[serde(serialize_with = "serialize_order_as_id")]
    raw_single_order: &'o mut RawSingleOrder,
    #[getset(get = "pub with_prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    limit: Option<Price>,
    #[getset(get = "pub with_prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    trigger_limit: Option<Price>,
    #[getset(get = "pub with_prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "crate::serde::amount_value::price::option")]
    absolute_trailing_limit: Option<Price>,
    #[getset(get = "pub with_prefix")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "PrimitiveValue::serialize_option_f64_str")]
    relative_trailing_limit: Option<Percent>,
    #[serde(flatten)]
    #[getset(get = "pub with_prefix")]
    #[serde(with = "crate::serde::order_validity::option")]
    #[serde(skip_serializing_if = "Option::is_none")]
    validity: Option<OrderValidity>,
}

#[derive(serde::Serialize)]
pub(crate) struct DeleteOrder {}

impl OrderChangeValidation<'_, '_, '_> {
    #[inline(always)]
    pub(crate) fn order_id(&self) -> &OrderId {
        use OrderChangeValidation::*;
        match self {
            Change(change) => change.order_id(),
            Delete(order) => order.id()
        }
    }
}

impl OrderChangeAction<'_, '_> {
    #[inline(always)]
    pub(crate) fn order_id(&self) -> &OrderId {
        use OrderChangeAction::*;
        match self {
            Change(change) => change.order_id(),
            Delete(order) => order.id()
        }
    }
}

impl<'o> OrderChange<'o> {
    pub fn from_order0(order: &'o mut ComdirectOrder<'_>) -> Self {
        let raw_single_order = match order.raw {
            RawOrder::SingleOrder(ref mut raw) => raw,
            RawOrder::CombinationOrder(ref mut raw) => &mut raw.sub_orders.0
        };
        Self::from_raw_single_order(raw_single_order)
    }

    pub fn from_order1(order: &'o mut ComdirectOrder<'_>) -> Self {
        let raw_single_order = match order.raw {
            RawOrder::SingleOrder(ref mut raw) => raw,
            RawOrder::CombinationOrder(ref mut raw) => &mut raw.sub_orders.1
        };
        Self::from_raw_single_order(raw_single_order)
    }

    fn from_raw_single_order(raw_single_order: &'o mut RawSingleOrder) -> Self {
        Self {
            raw_single_order,
            limit: None,
            trigger_limit: None,
            absolute_trailing_limit: None,
            relative_trailing_limit: None,
            validity: None,
        }
    }

    pub(crate) fn change_order(self) {
        self.limit
            .map(|limit| self.raw_single_order.limit = Some(limit));
        self.trigger_limit
            .map(|limit| self.raw_single_order.trigger_limit = Some(limit));
        self.absolute_trailing_limit
            .map(|limit| self.raw_single_order.absolute_trailing_limit = Some(limit));
        self.relative_trailing_limit
            .map(|limit| self.raw_single_order.relative_trailing_limit = Some(limit));
        self.validity
            .map(|validity| self.raw_single_order.validity = validity);
    }

    #[inline(always)]
    pub fn order_id(&self) -> &OrderId {
        &self.raw_single_order.id
    }

    option_builder_fn!(
        pub fn limit(Price)
        pub fn trigger_limit(Price)
        pub fn absolute_trailing_limit(Price)
        pub fn relative_trailing_limit(Percent)
        pub fn validity(OrderValidity)
    );
}

pub(crate) fn serialize_order_as_id<S>(order: &RawSingleOrder, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
    order.id.serialize(serializer)
}
