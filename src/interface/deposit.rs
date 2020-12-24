use super::*;

impl ApiClient {
    pub fn get_deposits(&self) -> Result<Vec<ComdirectDeposit>> {
        const URL: &str = url!("/brokerage/clients/user/v3/depots");
        let session = session_is_active!(self.session);

        Ok(
            self.make_get_session_request(URL, &session)
                .send()?
                .error_for_status()?
                .json::<JsonResponseValues<ComdirectDeposit>>()?
                .values
        )
    }

    //noinspection RsUnresolvedReference
    pub fn get_positions<'d>(&self, deposit: &'d ComdirectDeposit) -> Result<Vec<Position<'d>>> {
        let raw_positions = self.send_get_positions_request(deposit)?;
        let mut positions = Vec::with_capacity(raw_positions.len());

        for raw in raw_positions {
            positions.push(Position::from_raw(raw, deposit));
        }

        Ok(positions)
    }

    pub fn get_position<'d>(&self, deposit: &'d ComdirectDeposit, position_id: &PositionId) -> Result<Position<'d>> {
        let raw_position = self
            .send_get_position_request(deposit, position_id)?
            .json::<RawPosition>()?;
        Ok(Position::from_raw(raw_position, deposit))
    }

    pub fn update_position(&self, position: &mut Position) -> Result<()> {
        let response = self.send_get_position_request(position.deposit(), position.raw().id())?;
        position.update_from_response(response)?;
        Ok(())
    }

    pub fn get_deposit_transactions<'d>(&self, deposit: &'d ComdirectDeposit) -> Result<Vec<Transaction<'d>>> {
        self._get_deposit_transactions(deposit, None)
    }

    pub fn get_deposit_transactions_filtered<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: &TransactionFilterParameters) -> Result<Vec<Transaction<'d>>> {
        self._get_deposit_transactions(deposit, Some(filter_parameters))
    }

    #[inline(always)]
    fn send_get_positions_request(&self, deposit: &ComdirectDeposit) -> Result<Vec<RawPosition>> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/positions", url!("/brokerage/v3/depots"), deposit.id());

        Ok(
            self.make_get_session_request(&url, session)
                .query(&[("without-attr", "depot")])
                .send()?
                .error_for_status()?
                .json::<JsonResponseValues<RawPosition>>()?
                .values
        )
    }

    //noinspection RsUnresolvedReference
    #[inline(always)]
    fn send_get_position_request(&self, deposit: &ComdirectDeposit, position_id: &PositionId) -> Result<Response> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/positions/{}", url!("/brokerage/v3/depots"), deposit.id(), position_id.as_str());

        Ok(
            self.make_get_session_request(&url, session)
                .query(&[("without-attr", "depot")])
                .send()?
        )
    }

    //noinspection RsUnresolvedReference
    #[inline(always)]
    fn _get_deposit_transactions<'d>(&self, deposit: &'d ComdirectDeposit, filter_parameters: Option<&TransactionFilterParameters>) -> Result<Vec<Transaction<'d>>> {
        let raw_transactions = self.send_get_deposit_transactions_request(deposit, filter_parameters)?;
        let mut transactions = Vec::with_capacity(raw_transactions.len());

        for raw in raw_transactions {
            transactions.push(Transaction::from_raw(raw, deposit))
        }

        Ok(transactions)
    }

    #[inline(always)]
    fn send_get_deposit_transactions_request(&self, deposit: &ComdirectDeposit, filter_parameters: Option<&TransactionFilterParameters>) -> Result<Vec<RawTransaction>> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}/transactions", url!("/brokerage/v3/depots"), deposit.id());

        let mut request = self.make_get_session_request(&url, session)
            .query(&[("without-attr", "instrument")]);
        if let Some(filters) = filter_parameters {
            request = request.query(filters);
        }

        Ok(
            request.send()?
                .error_for_status()?
                .json::<JsonResponseValues<RawTransaction>>()?
                .values
        )
    }
}
