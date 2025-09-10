use validator::{Validate, ValidationError, ValidationErrors};

#[cfg(test)]
mod tests;

fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    if password.len() < 8 {
        return Err(ValidationError::new(
            "Password must be at least 8 characters long.",
        ));
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(ValidationError::new(
            "Password must contain at least one uppercase letter.",
        ));
    }
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err(ValidationError::new(
            "Password must contain at least one lowercase letter.",
        ));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ValidationError::new(
            "Password must contain at least one digit.",
        ));
    }
    if !password
        .chars()
        .any(|c| "!@#$%^&*()_+=|[]{{}};:'\",.<>/?~-".contains(c))
    {
        return Err(ValidationError::new(
            "Password must contain at least one special character.",
        ));
    }

    Ok(())
}

#[derive(Debug, Validate, Clone, Eq, Hash, PartialEq)]
pub struct Password {
    #[validate(custom(function = "validate_password_strength"))]
    pub password: String,
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        self.password.as_str()
    }
}

impl Password {
    pub fn parse(password: &str) -> Result<Self, ValidationErrors> {
        let password = Self {
            password: password.to_owned(),
        };
        password.validate()?;
        Ok(password)
    }
}
