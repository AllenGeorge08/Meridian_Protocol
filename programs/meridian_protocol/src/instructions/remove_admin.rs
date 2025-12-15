use anchor_lang::prelude::*;

use crate::errors::Errors;
use crate::states::*;

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[derive(Accounts)]
pub struct RemoveAdmin<'info> {
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
        bump = lending_pool.bump_admin_registry
    )]
    pub admin_registry: Box<Account<'info, AdminRegistry>>,
    pub system_program: Program<'info, System>,
}

impl<'info> RemoveAdmin<'info> {
    pub fn remove_admin(&mut self, admin: Pubkey) -> Result<()> {
        require!(
            self.authority.key() == self.lending_pool.owner.key(),
            Errors::OnlyAuthority
        );
        require!(
            self.admin_registry.admins.contains(&admin),
            Errors::AdminInvalid
        );

        let admin_index = self.admin_registry.admins.iter().position(|&a| a == admin);
        require!(admin_index.is_some(), Errors::AdminInvalid);

        self.remove_admin(admin)?;

        msg!("Admin removed : {}", admin);
        Ok(())
    }

    pub fn is_admin(&mut self, admin: Pubkey) -> bool {
        let _ = self.admin_registry.is_admin(admin);
        return false;
    }
}
