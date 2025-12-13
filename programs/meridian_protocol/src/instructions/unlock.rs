use crate::errors::Errors;
use crate::states::*;
use anchor_lang::prelude::*;

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[derive(Accounts)]
pub struct UnLockPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"meridian_pool",authority.key().as_ref()],
        bump = lending_pool.bump_lending_pool
    )]
    pub lending_pool: Box<Account<'info, LendingPool>>,
    pub system_program: Program<'info, System>,
}

impl<'info> UnLockPool<'info> {
    pub fn unlock_pool(&mut self) -> Result<()> {
        require!(
            self.authority.key() == self.lending_pool.owner.key(),
            Errors::OnlyAuthority
        );
        require!(self.lending_pool.is_locked, Errors::PoolAlreadyUnLocked);
        self.lending_pool.is_locked = false;
        msg!("Pool Unlocked");
        Ok(())
    }
}
