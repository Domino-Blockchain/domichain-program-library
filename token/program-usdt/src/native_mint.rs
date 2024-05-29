//! The Mint that represents the native token

/// There are 10^9 satomis in one DOMI
pub const DECIMALS: u8 = 9;

// The Mint for native DOMI Token accounts
domichain_program::declare_id!("So11111111111111111111111111111111111111112");

#[cfg(test)]
mod tests {
    use super::*;
    use domichain_program::native_token::*;

    #[test]
    fn test_decimals() {
        assert!(
            (satomis_to_domi(42) - crate::amount_to_ui_amount(42, DECIMALS)).abs() < f64::EPSILON
        );
        assert_eq!(
            domi_to_satomis(42.),
            crate::ui_amount_to_amount(42., DECIMALS)
        );
    }
}
