// use email_address::EmailAddress;
use serde::{Deserialize, Deserializer, Serialize};

use crate::models::validation;

#[derive(Serialize, Deserialize, Debug)]
pub struct NewSubscriber {
    #[serde(deserialize_with = "validation::validate_email_address")]
    email: String,
    #[serde(deserialize_with = "validation::validate_name")]
    name: String,
}

// The caller gets a shared reference to the inner string.
// This gives the caller **read-only** access,
// This means that once NewSubscriber has been parsed, it's values
// cannot later be changed to invalid ones.
impl NewSubscriber {
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_email(&self) -> &str {
        &self.email
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use quickcheck::Gen;
    use rand::{rngs::StdRng, SeedableRng};

    #[derive(Serialize, Deserialize)]
    pub struct SubscribeRequest {
        email: String,
        name: String,
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);
    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    // quickcheck will generate 100 valid emails to test.
    #[quickcheck_macros::quickcheck]
    // #[test]
    pub fn valid_email_is_parsed_succesfully(valid_email: ValidEmailFixture) -> bool {
        let body = SubscribeRequest {
            email: valid_email.0,
            // email: String::from("ursula_le_guin@gmail.com"),
            name: String::from("Ursula le Quin"),
        };
        let json_str = serde_json::to_string(&body).expect("Failed so serialize request.");
        let parsed_data = serde_json::from_str::<NewSubscriber>(&json_str);
        match parsed_data {
            Ok(_) => return true,
            Err(_) => return false,
        };
    }
}
