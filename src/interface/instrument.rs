use super::*;

impl ApiClient {
    pub fn get_instrument(&self, derivative: &Derivative) -> Result<Instrument> {
        let session = session_is_active!(self.session);
        let url = format!("{}/{}", url!("/brokerage/v1/instruments/"), derivative.as_ref());

        Ok(
            self.make_get_session_request(&url, session)
                .query(&[
                    ("with-attr", "derivativeData"),
                    ("with-attr", "fundDistribution"),
                ])
                .send()?
                .error_for_status()?
                .json::<JsonResponseValue<Instrument>>()?
                .values.0
        )
    }
}
