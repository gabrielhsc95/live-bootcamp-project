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
}
