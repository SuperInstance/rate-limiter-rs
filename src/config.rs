//! Configuration types shared across rate limiter implementations.

/// Errors that can occur during configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// A parameter was zero or negative where a positive value is required.
    InvalidParameter(&'static str),
}

impl core::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ConfigError::InvalidParameter(name) => {
                write!(f, "invalid parameter: '{name}' must be positive")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Validate that a value is positive (non-zero).
pub fn validate_positive(value: u64, name: &'static str) -> Result<(), ConfigError> {
    if value == 0 {
        Err(ConfigError::InvalidParameter(name))
    } else {
        Ok(())
    }
}

/// Validate that a float value is positive and finite.
pub fn validate_positive_f64(value: f64, name: &'static str) -> Result<(), ConfigError> {
    if value <= 0.0 || !value.is_finite() {
        Err(ConfigError::InvalidParameter(name))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_positive_ok() {
        assert!(validate_positive(1, "test").is_ok());
    }

    #[test]
    fn validate_positive_zero_fails() {
        assert!(validate_positive(0, "test").is_err());
    }

    #[test]
    fn validate_positive_f64_ok() {
        assert!(validate_positive_f64(1.0, "test").is_ok());
    }

    #[test]
    fn validate_positive_f64_zero_fails() {
        assert!(validate_positive_f64(0.0, "test").is_err());
    }

    #[test]
    fn validate_positive_f64_negative_fails() {
        assert!(validate_positive_f64(-1.0, "test").is_err());
    }

    #[test]
    fn validate_positive_f64_nan_fails() {
        assert!(validate_positive_f64(f64::NAN, "test").is_err());
    }

    #[test]
    fn validate_positive_f64_infinity_fails() {
        assert!(validate_positive_f64(f64::INFINITY, "test").is_err());
    }

    #[test]
    fn config_error_display() {
        let err = ConfigError::InvalidParameter("capacity");
        assert!(format!("{err}").contains("capacity"));
    }
}
