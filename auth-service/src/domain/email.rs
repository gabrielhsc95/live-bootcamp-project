use validator::{Validate, ValidationErrors};

#[cfg(test)]
mod tests;

#[derive(Debug, Validate, Clone, Eq, Hash, PartialEq)]
pub struct Email {
    #[validate(email)]
    pub email: String,
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.email.as_str()
    }
}

impl Email {
    pub fn parse(email: &str) -> Result<Self, ValidationErrors> {
        let email = Self {
            email: email.to_owned(),
        };
        email.validate()?;
        Ok(email)
    }

    pub fn new_no_validation(email: String) -> Self {
        // Use with care, since there is no validation, for example, from
        // parsing a User from a database, with the assumption if it is in
        // database it has been validated already.
        Self { email }
    }
}
