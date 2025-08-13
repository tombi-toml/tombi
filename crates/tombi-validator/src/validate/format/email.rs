use email_address::EmailAddress;

pub fn validate(value: &str) -> bool {
    EmailAddress::is_valid(value)
}
