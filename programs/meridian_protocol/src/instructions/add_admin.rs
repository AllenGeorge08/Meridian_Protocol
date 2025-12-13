use crate::errors::Errors;
use crate::states::*;
use anchor_lang::prelude::*;

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[derive(Accounts)]
pub struct AddAdmin<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"meridian_pool",authority.key().as_ref()],
        bump = lending_pool.bump_lending_pool
    )]
    pub lending_pool: Box<Account<'info, LendingPool>>,
    #[account(
        mut,
        seeds = [b"meridian_pool_admin_registry",lending_pool.key().as_ref()],
        bump
    )]
    pub admin_registry: Box<Account<'info, AdminRegistry>>,
    pub system_program: Program<'info, System>,
}

impl<'info> AddAdmin<'info> {
    pub fn add_admin(&mut self, admin: Pubkey) -> Result<()> {
        require!(
            self.authority.key() == self.lending_pool.owner.key(),
            Errors::OnlyAuthority
        );
        self.add_admin(admin)?;
        msg!("Admin added : {}", admin);
        Ok(())
    }

    pub fn is_admin(&mut self, admin: Pubkey) -> bool {
        let _ = self.admin_registry.is_admin(admin);
        return false;
    }
}
