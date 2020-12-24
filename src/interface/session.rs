use super::*;

impl ApiClient {
    pub fn new_session(&mut self) -> Result<()> {
        self.session = None;
        let session = self.create_session()?;
        self.session = Some(session);

        Ok(())
    }

    pub fn refresh_session(&mut self) -> Result<()> {
        let mut session = session_is_active!(*self.session.take());
        let refresh_session = self.acquire_oauth_token(GrantType::Refresh(&session.refresh_token))?;

        session.update(refresh_session);

        self.session = Some(session);

        Ok(())
    }

    pub fn end_session(&mut self) -> Result<()> {
        self.revoke_oauth_token()?;
        self.session = None;

        Ok(())
    }

    fn create_session(&mut self) -> Result<Session> {
        let pre_session = self.acquire_oauth_token(GrantType::Password)?;
        let mut session = self.acquire_session_status(pre_session)?;

        let tan_challenge = self.request_tan_challenge(&session, None)?;
        self.activate_tan(&session, tan_challenge)?;

        let secondary_session = self.acquire_oauth_token(GrantType::CdSecondary(&session.access_token))?;
        session.update(secondary_session);

        Ok(session)
    }

    fn acquire_oauth_token(&self, grant_type: GrantType) -> Result<PreSession> {
        const URL: &str = "https://api.comdirect.de/oauth/token";

        Ok(
            self.client
                .post(URL)
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .form(&self.make_oauth_params(grant_type))
                .send()?
                .error_for_status()?
                .json::<PreSession>()?
        )
    }

    fn make_oauth_params<'s>(&'s self, grant_type: GrantType<'s>) -> HashMap<&'static str, &'s str> {
        let mut params: HashMap<&str, &str> = HashMap::new();

        params.insert("client_id", self.client_id.as_str());
        params.insert("client_secret", self.client_secret.as_str());
        params.insert("grant_type", grant_type.as_str());

        self.add_oauth_params_grant_type(grant_type, &mut params);

        params
    }

    fn add_oauth_params_grant_type<'s>(&'s self, grant_type: GrantType<'s>, params: &mut HashMap<&'static str, &'s str>) {
        match grant_type {
            GrantType::Password => {
                params.insert("username", self.username.as_str());
                params.insert("password", self.password.as_str());
            }
            GrantType::CdSecondary(access_token) => {
                params.insert("token", access_token.as_str());
            }
            GrantType::Refresh(refresh_token) => {
                params.insert("refresh_token", refresh_token.as_str());
            }
        }
    }

    fn revoke_oauth_token(&self) -> Result<()> {
        const URL: &str = "https://api.comdirect.de/oauth/revoke";
        let session = session_is_active!(self.session);

        self.client
            .delete(URL)
            .bearer_auth(&session.access_token)
            .send()?
            .error_for_status()?;

        Ok(())
    }

    fn acquire_session_status(&self, pre_session: PreSession) -> Result<Session> {
        const URL: &str = url!("/session/clients/user/v1/sessions");
        let session_id = Self::make_session_id();

        let session_status = self.client
            .get(URL)
            .bearer_auth(&pre_session.access_token)
            .header("x-http-request-info", self.make_request_info(&session_id))
            .send()?
            .error_for_status()?
            .json::<(SessionStatus, )>()?
            .0;

        Ok(
            Session::from_pre_session(
                pre_session,
                session_id,
                session_status.take_session_uuid(),
            )
        )
    }

    #[inline(always)]
    fn make_session_id() -> SessionId {
        let mut rng = rand::thread_rng();
        let session_id: String = (0..32)
            .map(|_| {
                let char_id = rng.gen_range(0, HEX_CHARSET.len());
                HEX_CHARSET[char_id] as char
            })
            .collect();
        SessionId(session_id)
    }

    fn request_tan_challenge(&self, session: &Session, desired_tan_type: Option<TanChallengeType>)
        -> Result<TanChallenge> {
        let url = format!("{}/{}/validate", url!("/session/clients/user/v1/sessions"), session.session_uuid.as_str());

        let response = self
            .make_request_tan_challenge_request_builder(url, session, desired_tan_type)
            .send()?
            .error_for_status()?;

        let tan_challenge = self.check_tan_challenge(
            Self::extract_tan_challenge(response.headers())?,
            desired_tan_type,
            session,
        )?;

        Ok(tan_challenge)
    }

    fn make_request_tan_challenge_request_builder(&self, url: String, session: &Session, desired_tan_type: Option<TanChallengeType>)
        -> RequestBuilder {
        let data = format!(
            r#"{{
                "identifier": "{}",
                "sessionTanActive":true,
                "activated2FA":true
            }}"#,
            session.session_uuid.as_str()
        );

        let mut request_builder = self
            .make_post_session_request(&url, session)
            .body(data);

        if let Some(tan_type) = desired_tan_type {
            request_builder = request_builder.header(
                "x-once-authentication-info",
                tan_type.to_authentication_info(),
            );
        }

        request_builder
    }

    fn check_tan_challenge(&self, current_tan_challenge: TanChallenge, desired_tan_type: Option<TanChallengeType>, session: &Session)
        -> Result<TanChallenge> {
        match current_tan_challenge.typ() {
            TanChallengeType::PushTan => Ok(current_tan_challenge),
            _ => Ok(self.handle_unsupported_tan_type(
                &current_tan_challenge,
                desired_tan_type,
                session,
            )?)
        }
    }

    fn handle_unsupported_tan_type(&self, current_tan_challenge: &TanChallenge, desired_tan_type: Option<TanChallengeType>, session: &Session)
        -> Result<TanChallenge> {
        match desired_tan_type {
            Some(desired) => match desired == *current_tan_challenge.typ() {
                true => Err(Error::UnsupportedTanType),
                false => Err(Error::UnexpectedTanType)
            },
            None => match current_tan_challenge.available_types().contains(&TanChallengeType::PushTan) {
                true => Ok(self.request_tan_challenge(&session, Some(TanChallengeType::PushTan))?),
                false => Err(Error::UnsupportedTanType)
            }
        }
    }

    fn activate_tan(&self, session: &Session, tan_challenge: TanChallenge) -> Result<()> {
        let tan = Self::ask_for_tan(&tan_challenge)?;
        let url = format!("{}/{}", url!("/session/clients/user/v1/sessions"), session.session_uuid.as_str());

        let session_status = self
            .activate_tan_request_builder(url, session, tan_challenge, tan)
            .send()?
            .error_for_status()?
            .json::<SessionStatus>()?;

        match session_status.tan_is_active() {
            true => Ok(()),
            false => Err(Error::CouldNotCreateSession)
        }
    }

    fn activate_tan_request_builder(&self, url: String, session: &Session, tan_challenge: TanChallenge, tan: String)
        -> RequestBuilder {
        let tan_header = Self::make_x_authentication_info_header(&tan_challenge);
        let data = format!(
            r#"{{
                "identifier": "{}",
                "sessionTanActive":true,
                "activated2FA":true
            }}"#,
            session.session_uuid.as_str()
        );

        let request_builder = self
            .make_patch_session_request(&url, session)
            .header(tan_header.0, tan_header.1)
            .body(data);

        match tan_challenge.typ() {
            TanChallengeType::PushTan => request_builder,
            TanChallengeType::Free => unreachable!("got TanType Free while creating a session"),
            _ => request_builder.header("x-once-authentication", tan),
        }
    }

    #[cfg(any(test, feature = "test"))]
    fn ask_for_tan(_: &TanChallenge) -> StdResult<String, !> {
        std::thread::sleep(std::time::Duration::from_secs(10));
        Ok(String::from('0'))
    }

    #[cfg(not(any(test, feature = "test")))]
    fn ask_for_tan(tan_challenge: &TanChallenge) -> Result<String> {
        Self::print_tan_challenge_type_msg(tan_challenge);

        let mut tan = String::new();
        std::io::stdin()
            .read_line(&mut tan)?;

        Ok(tan)
    }

    #[cfg_attr(any(test, feature = "test"), allow(dead_code))]
    fn print_tan_challenge_type_msg(tan_challenge: &TanChallenge) {
        use TanChallengeType::*;
        match tan_challenge.typ() {
            PushTan => print!("Please open your PhotoTan App and activate the PushTan."),
            Free => unreachable!("got TanType Free while creating a session"),
            MobileTan if tan_challenge.challenge().is_some() => println!(
                "Please call '{}' and input the TAN.",
                tan_challenge.challenge().as_ref().unwrap()
            ),
            t =>
                log::warn!("Could not print a message for TanChallengeType ({:?}), since it's not supported yet!", t)
        }
        println!(" Then press enter.")
    }
}
