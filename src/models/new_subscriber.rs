// use email_address::EmailAddress;
use serde::{Deserialize, Deserializer, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use validator::validate_email;

#[derive(Serialize, Deserialize, Debug)]
pub struct NewSubscriber {
    #[serde(deserialize_with = "validate_email_address")]
    email: String,
    #[serde(deserialize_with = "validate_name")]
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

fn validate_email_address<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    let is_valid = validate_email(&s);
    if is_valid {
        return Ok(s);
    } else {
        return Err(serde::de::Error::custom("Not a valid email address."));
    };
}

fn validate_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;

    // A grapheme is defined by the Unicode standard as a "user-perceived"
    // character: `å` is a single grapheme, but it is composed of two characters
    // (`a` and ``).
    //
    // `graphemes` returns an iterator over the graphemes in the input `s`.
    // `true` specifies that we want to use the extended grapheme definition set,
    // the recommended one.
    if s.graphemes(true).count() > 256 {
        return Err(serde::de::Error::custom(
            "Name length exceeds 256 characters.",
        ));
    }
    if s.trim().is_empty() {
        return Err(serde::de::Error::custom("Name is empty string."));
    }
    // Iterate over all characters in the input `s` to check if any of them matches
    // one of the characters in the forbidden array.
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    if s.chars().any(|g| forbidden_characters.contains(&g)) {
        return Err(serde::de::Error::custom(
            "Name contains forbidden character(s).",
        ));
    }
    Ok(s)
}
