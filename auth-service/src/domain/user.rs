use validator::{Validate, ValidationErrors};

use crate::domain::{Email, Password};

#[derive(Debug)]
pub enum ParseErrors {
    InvalidEmail,
    InvalidPassword,
}

#[derive(Debug, Clone)]
pub struct User {
    email: Email,
    password: Password,
    requires_2fa: bool,
}

impl User {
    fn new(email: Email, password: Password, requires_2fa: bool) -> Self {
        Self {
            email,
            password,
            requires_2fa,
        }
    }

    pub fn email_str(&self) -> &str {
        self.email.as_ref()
    }

    pub fn email(&self) -> Email {
        self.email.clone()
    }

    pub fn password_str(&self) -> &str {
        self.password.as_ref()
    }

    pub fn password(self) -> Password {
        self.password.clone()
    }

    pub fn requires_2fa(&self) -> bool {
        self.requires_2fa
    }

    pub fn parse(
        email: String,
        password: String,
        requires_2fa: bool,
    ) -> Result<Self, ValidationErrors> {
        let email = Email::parse(&email)?;
        let password = Password::parse(&password)?;

        Ok(Self::new(email, password, requires_2fa))
    }
}
