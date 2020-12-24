use super::*;

impl ApiClient {
    pub fn get_marketplaces(&self) -> Result<Vec<MarketPlace>> {
        self._get_marketplaces(None)
    }

    pub fn get_marketplaces_filtered(&self, filter_parameters: &MarketPlaceFilterParameters)
        -> Result<Vec<MarketPlace>> {
        self._get_marketplaces(Some(filter_parameters))
    }

    #[inline(always)]
    fn _get_marketplaces(&self, filter_parameters: Option<&MarketPlaceFilterParameters>)
        -> Result<Vec<MarketPlace>> {
        const URL: &str = url!("/brokerage/v3/orders/dimensions");
        let session = session_is_active!(self.session);

        let mut request = self.make_get_session_request(URL, session);
        if let Some(filters) = filter_parameters {
            request = request.query(filters)
        }

        let response = request.send()?
            .error_for_status()?;

        let json = response.json::<JsonResponseMarketplaces>()?;
        Ok(json.market_places())
    }

    pub fn get_orders<'d>(&self, deposit: &'d ComdirectDeposit) -> Result<Vec<Order<'d>>> {
        self._get_orders(deposit, None)
    }

    pub fn get_orders_filtered<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: &OrderFilterParameters)
        -> Result<Vec<Order<'d>>> {
        self._get_orders(deposit, Some(filter_parameters))
    }

    //noinspection RsUnresolvedReference
    #[inline(always)]
    fn _get_orders<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: Option<&OrderFilterParameters>)
        -> Result<Vec<Order<'d>>> {
        let session = session_is_active!(&self.session);
        let url = format!("{}/{}/v3/orders", url!("/brokerage/depots"), deposit.id());

        let mut request = self.make_get_session_request(&url, session);
        if let Some(filters) = filter_parameters {
            request = request.query(filters);
        }

        let response = request.send()?
            .error_for_status()?;

        let json = response.json::<JsonResponseValues<RawOrder>>()?;

        let mut orders = Vec::with_capacity(json.values.len());
        for raw in json.values {
            orders.push(Order::from_raw(raw, deposit));
        }

        Ok(orders)
    }

    pub fn get_order<'d>(&self, deposit: &'d ComdirectDeposit, order_id: &OrderId) -> Result<Order<'d>> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}", url!("/brokerage/v3/orders"), order_id.as_str());

        let response = self.make_get_session_request(&url, session)
            .send()?
            .error_for_status()?;

        let raw = response.json::<RawOrder>()?;
        Ok(Order::from_raw(raw, deposit))
    }

    pub fn order_cost_indication<'o, 'd, 'i, 'm>(&self, order_outline: &'o OrderOutline<'d, 'i, 'm>) -> Result<CostIndication<'o, 'd, 'i, 'm>> {
        let raw = self._order_cost_indication(order_outline)?;
        let cost_indication = CostIndication::from_raw(raw, order_outline);
        Ok(cost_indication)
    }

    pub(super) fn _order_cost_indication<O: Serialize>(&self, outline: &O) -> Result<RawCostIndication> {
        const URL: &str = url!("/brokerage/v3/orders/costindicationexante");
        let session = session_is_active!(self.session);

        Ok(
            self.make_post_session_request(URL, session)
                .json(outline)
                .send()?
                .error_for_status()?
                .json::<JsonResponseValue<RawCostIndication>>()?
                .values.0
        )
    }

    pub fn pre_validate_order_outline(&self, order_outline: &OrderOutline) -> Result<()> {
        const URL: &str = url!("/brokerage/v3/orders/prevalidation");
        let session = session_is_active!(self.session);

        self.make_post_session_request(URL, session)
            .json(order_outline)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    pub fn place_order<'d>(&self, order_outline: &OrderOutline<'d, '_, '_>) -> Result<Order<'d>> {
        let tan_challenge = self.validate_outline(order_outline)?;
        let order = self.place_order_outline(order_outline, tan_challenge)?;
        Ok(order)
    }

    pub(super) fn validate_outline<O: Serialize>(&self, outline: &O) -> Result<TanChallenge> {
        const URL: &str = url!("/brokerage/v3/orders/validation");
        let session = session_is_active!(self.session);

        let response = self.make_post_session_request(URL, session)
            .json(outline)
            .send()?
            .error_for_status()?;

        let tan_challenge = ApiClient::extract_tan_challenge(response.headers())?;
        tan_is_free!(tan_challenge);

        Ok(tan_challenge)
    }

    fn place_order_outline<'d>(&self, order_outline: &OrderOutline<'d, '_, '_>, tan_challenge: TanChallenge) -> Result<Order<'d>> {
        let raw_order = self.place_outline(order_outline, tan_challenge)?;
        let order = Order::from_raw(raw_order, order_outline.deposit());
        Ok(order)
    }

    pub(super) fn place_outline<O: Serialize>(&self, outline: &O, tan_challenge: TanChallenge) -> Result<RawOrder> {
        const URL: &str = url!("/brokerage/v3/orders");
        let session = session_is_active!(self.session);
        let tan_header = Self::make_x_authentication_info_header(&tan_challenge);

        let response = self.make_post_session_request(URL, session)
            .header(tan_header.0, tan_header.1)
            .json(outline)
            .send()?
            .error_for_status()?;

        let raw_order = response.json::<RawOrder>()?;
        Ok(raw_order)
    }

    pub fn pre_validate_order_change(&self, order_change: &OrderChange) -> Result<()> {
        let validation = OrderChangeValidation::Change(order_change);
        self._pre_validate_order_change(validation)
    }

    pub fn pre_validate_order_deletion(&self, order: &Order) -> Result<()> {
        let validation = OrderChangeValidation::Delete(order);
        self._pre_validate_order_change(validation)
    }

    #[inline(always)]
    fn _pre_validate_order_change(&self, change_validation: OrderChangeValidation) -> Result<()> {
        let session = session_is_active!(self.session);
        let url = format!(
            "{}/{}/prevalidation",
            url!("/brokerage/v3/orders"), change_validation.order_id()
        );

        Self::make_order_change_body(
            self.make_post_session_request(&url, session),
            &change_validation,
        )
            .send()?
            .error_for_status()?;
        Ok(())
    }

    pub fn order_change_cost_indication<'oc, 'o>(&self, order_change: &'oc OrderChange<'o>) -> Result<ChangeCostIndication<'oc, 'o, '_>> {
        let validation = OrderChangeValidation::Change(order_change);
        self._order_change_cost_indication(validation)
    }

    // FIXME: Currently this interface does not work / is not available
    // I already contacted the Comdirect support
    // #[inline(always)]
    // pub fn order_deletion_cost_indication<'o, 'd>(&self, order: &'o ComdirectOrder<'d>) -> Result<ChangeCostIndication<'_, 'o, 'd>, Error> {
    //     let validation = OrderChangeValidation::Delete(order);
    //     self._order_change_cost_indication(validation)
    // }
    #[inline(always)]
    fn _order_change_cost_indication<'oc, 'o, 'd>(&self, change_validation: OrderChangeValidation<'o, 'd, 'oc>) -> Result<ChangeCostIndication<'oc, 'o, 'd>> {
        use OrderChangeValidation::*;
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/costindicationexante", url!("/brokerage/v3/orders"), change_validation.order_id());

        let response = Self::make_order_change_body(
            self.make_post_session_request(&url, session),
            &change_validation,
        )
            .send()?
            .error_for_status()?;

        let raw = response.json::<JsonResponseValue<RawCostIndication>>()?.values.0;
        let cost_indication = match change_validation {
            Change(order_change) => ChangeCostIndication::Change { order_change, raw },
            Delete(order) => ChangeCostIndication::Delete { order, raw }
        };

        Ok(cost_indication)
    }

    pub fn change_order(&self, order_change: OrderChange) -> Result<()> {
        let tan_challenge = self.validate_order_change(&order_change)?;
        let action = OrderChangeAction::Change(order_change);
        self._change_order(action, tan_challenge)
    }

    pub fn delete_order<'d>(&self, order: Order<'d>) -> StdResult<(), (Error, Order<'d>)> {
        macro_rules! map_err {
            ($expr:expr) => {
                match $expr {
                    Ok(o) => o,
                    Err(e) => return Err((e, order))
                }
            };
        }

        let tan_challenge = map_err!(self.validate_order_deletion(&order));
        let action = OrderChangeAction::Delete(&order);
        map_err!(self._change_order(action, tan_challenge));
        Ok(())
    }

    fn validate_order_change(&self, order_change: &OrderChange) -> Result<TanChallenge> {
        let validation = OrderChangeValidation::Change(order_change);
        self._validate_order_change(validation)
    }

    fn validate_order_deletion(&self, order: &Order) -> Result<TanChallenge> {
        let validation = OrderChangeValidation::Delete(order);
        self._validate_order_change(validation)
    }

    #[inline(always)]
    fn _validate_order_change(&self, change_validation: OrderChangeValidation) -> Result<TanChallenge> {
        use OrderChangeValidation::*;
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/validation", url!("/brokerage/v3/orders"), change_validation.order_id());

        let mut request = self.make_post_session_request(&url, session);
        match change_validation {
            Change(order_change) => request = request.json(order_change),
            Delete(_) => request = request.json(&DeleteOrder {})
        }

        let response = request
            .send()?
            .error_for_status()?;
        let tan_challenge = ApiClient::extract_tan_challenge(response.headers())?;
        tan_is_free!(tan_challenge);

        Ok(tan_challenge)
    }

    #[inline(always)]
    fn _change_order(&self, change_action: OrderChangeAction, tan_challenge: TanChallenge) -> Result<()> {
        use OrderChangeAction::*;
        let session = session_is_active!(self.session);
        let url = format!("{}/{}", url!("/brokerage/v3/orders"), change_action.order_id());
        let tan_header = Self::make_x_authentication_info_header(&tan_challenge);

        let request = match change_action {
            Change(ref order_change) => {
                self
                    .make_patch_session_request(&url, session)
                    .json(order_change)
            }
            Delete(_) => {
                self
                    .make_delete_session_request(&url, session)
                    .json(&DeleteOrder {})
            }
        };

        request
            .header(tan_header.0, tan_header.1)
            .send()?
            .error_for_status()?;

        if let Change(order_change) = change_action {
            order_change.change_order();
        }
        Ok(())
    }

    #[inline(always)]
    fn make_order_change_body(request: RequestBuilder, change_validation: &OrderChangeValidation) -> RequestBuilder {
        use OrderChangeValidation::*;
        match change_validation {
            Change(order_change) => request.json(order_change),
            Delete(_) => request.json(&DeleteOrder {})
        }
    }
}
