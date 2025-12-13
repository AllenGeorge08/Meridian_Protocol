use anchor_lang::prelude::*;

#[error_code]
pub enum Errors{
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
    PoolAlreadyUnLocked
}