use anchor_lang::prelude::*;

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[error_code]
pub enum Errors {
    #[msg("Only Pool Owner Can Add Or Remove Admin")]
    OnlyAuthority,
    #[msg("Max Admins Reached, Cannot Add More")]
    MaxAdmins,
    #[msg("Admin already exists")]
    AdminAlreadyExists,
    #[msg("Admin doesn't exist , can't remove")]
    AdminInvalid,
    #[msg("Only Admin Can Carry this Operation")]
    OnlyAdmin,
    #[msg("Pool Is Already Locked")]
    PoolAlreadyLocked,
    #[msg("Pool Is Already  UnLocked")]
    PoolAlreadyUnLocked,
    #[msg("Null Deposits Not Allowed")]
    NullDepositNotAllowed,
    #[msg("Pool Locked")]
    PoolLocked,
    #[msg("Only the lender can withdraw")]
    InvalidUser,
    #[msg("Asset Not Verified, Cannot deposit")]
    AssetNotVerified,
    #[msg("Asset already sent for verification")]
    AssetAlreadySentForVerification,
    #[msg("Oracle Price Is Stale")]
    StaleOracle,
    #[msg("Invalid Price")]
    InvalidPrice,
    #[msg("Incorrect Verification ID")]
    IncorrectVerificationId,
    #[msg("Cannot collect collateral, asset deposited")]
    CannotCollectCollateral,
    #[msg("Amount Not Enough to Repay")]
    RepayAmountNotEnough,
    #[msg("Loan Cannot Be Repaid")]
    CannotRepayLoan,
}
