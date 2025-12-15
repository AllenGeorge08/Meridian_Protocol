use crate::errors::Errors;
use crate::states::*;
use anchor_lang::prelude::*;

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[derive(Accounts)]
pub struct MockOracle<'info> {
    #[account(mut)]
    pub owner_oracle: Signer<'info>,
    #[account(
        mut,
        seeds = [b"meridian_pool",lending_pool.owner.as_ref()],
        bump = lending_pool.bump_lending_pool
    )]
    pub lending_pool: Box<Account<'info, LendingPool>>,
    #[account(
        mut,
        seeds = [b"meridian_pool_admin_registry",lending_pool.key().as_ref()],
        bump = lending_pool.bump_admin_registry
    )]
    pub admin_registry: Box<Account<'info, AdminRegistry>>,
    #[account(
        mut,
        seeds = [b"meridian_mock_oracle",lending_pool.key().as_ref()],
        bump = mock_oracle.bump
    )]
    pub mock_oracle: Box<Account<'info, MockOracleState>>,
    pub system_program: Program<'info, System>,
}

impl<'info> MockOracle<'info> {
    pub fn update_oracle_values(&mut self, price: i64, exponent: i32) -> Result<(i64, i32)> {
        require!(
            self.owner_oracle.key() == self.lending_pool.owner
                || self.admin_registry.is_admin(self.owner_oracle.key()),
            Errors::OnlyAuthority
        );
        self.mock_oracle.price = price;
        self.mock_oracle.exponent = exponent;
        self.mock_oracle.last_updated = Clock::get()?.unix_timestamp;
        msg!("Updated Mock Oracle Values");

        Ok((price, exponent))
    }

    pub fn update_oracle_admin(&mut self, new_admin: Pubkey) -> Result<()> {
        require!(
            self.owner_oracle.key() == self.lending_pool.owner
                || self.admin_registry.is_admin(self.owner_oracle.key()),
            Errors::OnlyAuthority
        );
        require!(self.admin_registry.is_admin(new_admin), Errors::OnlyAdmin);
        self.mock_oracle.admin = new_admin;
        Ok(())
    }
}
