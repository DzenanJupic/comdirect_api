use super::*;

impl ApiClient {
    pub fn get_quote<'d>(&self, outline: &QuoteOutline<'d, '_, '_>) -> Result<Quote<'d>> {
        let (quote_ticket, tan_challenge) = self.validate_quote_outline(&outline)?;
        self.validate_quote_tan(&quote_ticket, tan_challenge)?;
        let quote = self.place_quote_outline(&outline, quote_ticket)?;

        log::info!("TEST");

        Ok(quote)
    }

    fn validate_quote_outline(&self, outline: &QuoteOutline) -> Result<(QuoteTicket, TanChallenge)> {
        const URL: &str = url!("/brokerage/v3/quoteticket");
        let session = session_is_active!(self.session);

        let response = self.make_post_session_request(URL, session)
            .json(outline)
            .send()?
            .error_for_status()?;

        let tan_challenge = ApiClient::extract_tan_challenge(response.headers())?;
        let quote_ticket = response.json::<QuoteTicket>()?;

        Ok((quote_ticket, tan_challenge))
    }

    fn validate_quote_tan(&self, quote_ticket: &QuoteTicket, tan_challenge: TanChallenge) -> Result<()> {
        match tan_challenge.typ() {
            TanChallengeType::Free => {}
            _ => return Err(Error::UnexpectedTanType)
        }

        let session = session_is_active!(self.session);
        let url = format!("{}/{}", url!("/brokerage/v3/quoteticket"), quote_ticket.id());
        let tan_header = Self::make_x_authentication_info_header(&tan_challenge);

        self.make_patch_session_request(&url, session)
            .header("x-once-authentication", "TAN_FREI")
            .header(tan_header.0, tan_header.1)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    fn place_quote_outline<'d>(&self, outline: &QuoteOutline<'d, '_, '_>, quote_ticket: QuoteTicket) -> Result<Quote<'d>> {
        const URL: &str = url!("/brokerage/v3/quotes");
        let session = session_is_active!(self.session);

        let response = self.make_post_session_request(URL, session)
            .json(outline)
            .send()?
            .error_for_status()?;

        let raw_quote = response.json::<RawQuote>()?;
        let quote = Quote::from_raw(raw_quote, quote_ticket, outline.deposit());

        Ok(quote)
    }

    pub fn quote_order_cost_indication<'o, 'd>(&self, order_outline: &'o QuoteOrderOutline<'d>) -> Result<CostIndication<'o, 'd, '_, '_>> {
        let _raw = dbg!(self._order_cost_indication(order_outline)?);
        unimplemented!()
        // let cost_indication = CostIndication::from_raw(raw, order_outline);
        // Ok(cost_indication)
    }

    pub fn place_quote_order<'d>(&self, quote_order_outline: QuoteOrderOutline<'d>) -> Result<Order<'d>> {
        let tan_challenge = self.validate_outline(&quote_order_outline)?;
        let order = self.place_quote_order_outline(&quote_order_outline, tan_challenge)?;
        Ok(order)
    }

    fn place_quote_order_outline<'d>(&self, quote_order_outline: &QuoteOrderOutline<'d>, tan_challenge: TanChallenge) -> Result<Order<'d>> {
        let raw_order = self.place_outline(quote_order_outline, tan_challenge)?;
        let order = Order::from_raw(raw_order, quote_order_outline.deposit());
        Ok(order)
    }
}
