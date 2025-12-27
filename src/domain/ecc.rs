//! Domain Model - ECC (Error Correction Code) Management
//!
//! This module defines ECC policies and status types,
//! independent of implementation details.

/// Policy for ECC handling during operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EccPolicy {
    /// Use hardware ECC (chip's internal ECC)
    #[default]
    Hardware,
    /// Disable hardware ECC for raw access
    Disabled,
    /// Use software ECC (for chips without hardware ECC)
    Software,
}

impl EccPolicy {
    /// Returns true if ECC should be enabled
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::Disabled)
    }

    /// Returns true if using hardware ECC
    pub fn is_hardware(&self) -> bool {
        matches!(self, Self::Hardware)
    }
}

/// Status of ECC operation after a read
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EccStatus {
    /// No errors detected
    NoError,
    /// Errors detected and corrected
    Corrected {
        /// Number of bits corrected
        bit_flips: u8,
    },
    /// Errors detected but uncorrectable
    Uncorrectable,
    /// ECC not available or disabled
    NotAvailable,
}

impl EccStatus {
    /// Returns true if data is valid (no error or corrected)
    pub fn is_valid(&self) -> bool {
        matches!(
            self,
            Self::NoError | Self::Corrected { .. } | Self::NotAvailable
        )
    }

    /// Returns true if there were bit flips that got corrected
    pub fn had_corrections(&self) -> bool {
        matches!(self, Self::Corrected { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecc_policy_helpers() {
        let hw = EccPolicy::Hardware;
        assert!(hw.is_enabled());
        assert!(hw.is_hardware());

        let sw = EccPolicy::Software;
        assert!(sw.is_enabled());
        assert!(!sw.is_hardware());

        let disabled = EccPolicy::Disabled;
        assert!(!disabled.is_enabled());
        assert!(!disabled.is_hardware());
    }

    #[test]
    fn test_ecc_status_helpers() {
        let no_error = EccStatus::NoError;
        assert!(no_error.is_valid());
        assert!(!no_error.had_corrections());

        let corrected = EccStatus::Corrected { bit_flips: 2 };
        assert!(corrected.is_valid());
        assert!(corrected.had_corrections());

        let uncorrectable = EccStatus::Uncorrectable;
        assert!(!uncorrectable.is_valid());
        assert!(!uncorrectable.had_corrections());

        let not_available = EccStatus::NotAvailable;
        assert!(not_available.is_valid()); // Considered valid as we can't tell
        assert!(!not_available.had_corrections());
    }
}
