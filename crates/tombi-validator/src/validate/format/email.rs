use email_address::EmailAddress;

pub fn validate_format(value: &str) -> bool {
    EmailAddress::is_valid(value)
}
