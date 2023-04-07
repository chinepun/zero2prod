use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{Secret, ExposeSecret};

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    username: Secret<String>,
    password: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String, 
        sender: SubscriberEmail,
        username: Secret<String>,
        password: Secret<String>,
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder()
            .timeout(timeout)
            .build()
            .unwrap();
        Self {
            http_client: http_client,
            base_url,
            sender,
            username,
            password,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/messages", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject: subject,
            html_body: html_content,
            text_body: text_content,
        };
        self.http_client
            .post(&url)
            .basic_auth(
                self.username.expose_secret(), 
                Some(self.password.expose_secret())
            )
            .form(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

}

#[derive(serde::Serialize)]
// #[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claims::{assert_ok, assert_err};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use wiremock::matchers::any;
    use wiremock::Request;
    use wiremock::matchers::{header, path, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Generate a random email subject
    fn subject() -> String {
            Sentence(1..2).fake()
    }
    /// Generate a random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }
    /// Generate a random subscriber email
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }
    /// Get a test instance of `EmailClient`.
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url, 
            email(), 
            Secret::new(Faker.fake()), 
            Secret::new(Faker.fake()),
            std::time::Duration::from_millis(200),
        )
    }
    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // Try to parse the body as a JSON value
            let result: Result<serde_json::Value, _> =
                serde_urlencoded::from_bytes::<serde_json::Value>(&request.body);
            if let Ok(body) = result {
                dbg!(&body);
                // Check that all the mandatory fields are populated
                // without inspecting the field values
                body.get("from").is_some()
                    && body.get("to").is_some()
                    && body.get("subject").is_some()
                    && body.get("html_body").is_some()
                    && body.get("text_body").is_some()
            } else {
            // If parsing failed, do not match the request
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request()
    {
        let mock_server = MockServer::start().await;
        // let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .and(header("Content-Type", "application/x-www-form-urlencoded"))
            .and(path("/messages"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;


        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;
            
        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
        assert_ok!(outcome);
    }

    #[tokio::test] 
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mock_server = MockServer::start().await;
        // let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();

        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            // Not a 200 anymore!
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Assert
        claims::assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        let response = ResponseTemplate::new(200)
            .set_delay(std::time::Duration::from_secs(180));// 3 mins
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }
}
