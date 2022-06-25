use reqwest::*;

pub struct CompanionAPI {
    email: String,
    password: String,
    client: Client,
}

impl CompanionAPI {
    /// Constructs a new `CompanionAPI`.
    ///
    /// # Examples
    ///
    /// ```
    /// use companionapi::CompanionAPI;
    ///
    /// let companion_api = CompanionAPI::new('some@email.com', 'somePassword');
    /// ```
    pub fn new(email: &str, password: &str) -> Self {
        let custom = redirect::Policy::custom(|attempt| {
            eprintln!("{}, Location: {:?}", attempt.status(), attempt.url());
            if attempt.url().path() == "/p/juno/login" {
                attempt.stop()
            } else {
                attempt.follow()
            }
        });

        Self {
            email: email.to_string(),
            password: password.to_string(),
            client: reqwest::Client::builder().redirect(custom).build().unwrap(),
        }
    }

    /// Logins to the CompanionAPI.
    ///
    /// # Examples
    ///
    /// ```
    /// use companionapi::CompanionAPI;
    ///
    /// let companion_api = CompanionAPI::new('some@email.com', 'somePassword');
    /// companion_api.login();
    /// ```
    pub async fn login(&self) -> Result<()> {
        /// Login flow
        /// 1. GET | https://accounts.ea.com/connect/auth?locale=en_US&state=bf4&redirect_uri=https%3A%2F%2Fbattlelog.battlefield.com%2Fsso%2F%3Ftokentype%3Dcode&response_type=code&client_id=battlelog&display=web%2Flogin
        /// 2. GET | Go to the location of the step 1
        /// 3. GET | Go to the location of the step 2 (save response cookies for the next queries)
        /// 4. GET | Go to the location of the step 3 (parse the #cid value from the form)
        /// 5. POST | Form to the location of the step 3 with the following parameters (save response cookies for the next queries)
        /// ```
        /// email: <emailAddress>
        /// regionCode: <regionCode: for example `FI`>
        /// phoneNumber: 
        /// password: <password>
        /// _eventId: submit
        /// cid: <cid: from the step 4 HTML response>
        /// showAgeUp: true
        /// thirdParyCaptchaResponse:
        /// loginMethod: emailPassword
        /// _rememberMe: on
        /// rememberMe: on
        /// ```
        /// If the response contains Location header
        ///     6.1 GET | Go to the location of the step 5 (this page comes if the login failed or if you need to for example accept terms of service)
        ///     7.1 If the HTML contains "juno/tosUpdate"
        /// 8.1 POST | Form to the location of the step 5
        /// ```
        /// _readAccept: on
        /// readAccept: on
        /// _eventId: accept
        /// ```
        ///     Else, login failed
        ///
        /// Finish login
        /// 1. GET | https://accounts.ea.com/connect/auth?initref_replay=false&display=web%2Flogin&response_type=code&redirect_uri=https%3A%2F%2Fbattlelog.battlefield.com%2Fsso%2F%3Ftokentype%3Dcode&locale=en_US&client_id=battlelog&fid=<fid from step 1 Location>
        let login_page = self.client
            .get("https://accounts.ea.com/connect/auth?locale=en_US&state=bf4&redirect_uri=https%3A%2F%2Fbattlelog.battlefield.com%2Fsso%2F%3Ftokentype%3Dcode&response_type=code&client_id=battlelog&display=web%2Flogin")
            .send()
            .await?;

        let location = login_page.headers().get("Location").unwrap();
        let login_qurey = login_page.url().query().unwrap();
        eprintln!("Login response query {:?}", login_qurey);

        Ok(())
    }

    // pub fn login_to_companion(&self) {
    //     ///
    //     /// Get the companion login token
    //     /// 1. GET | https://accounts.ea.com/connect/auth?client_id=sparta-companion-web&response_type=code&prompt=none&redirect_uri=nucleus:rest
    //     ///
    // }
}
