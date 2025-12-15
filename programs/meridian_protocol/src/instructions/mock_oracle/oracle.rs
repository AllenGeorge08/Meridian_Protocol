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
    pub const MAX_AGE: i64 = 100; //100 seconds is the maximum age for the oracle

    pub fn log_state(&mut self) {
        println!("Gold price (In troy ounce) : {} ", self.mock_oracle.price);
        println!("Gold exponent  : {} ", self.mock_oracle.exponent);
        println!("Last Updated at : {} ", self.mock_oracle.last_updated);
    }

    pub fn update_price_mock_oracle(&mut self, price: i64, exponent: i32) -> Result<()> {
        require!(self.is_admin(self.owner_oracle.key()), Errors::OnlyAdmin);
        let oracle = &mut self.mock_oracle;
        oracle.price = price;
        oracle.exponent = exponent;
        oracle.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn get_price_no_older_than(&mut self, max_age: i64) -> Result<(i64, i32)> {
        let current_time = Clock::get()?.unix_timestamp;
        let last_updated = self.mock_oracle.last_updated;
        let price = self.mock_oracle.price;
        let exponent = self.mock_oracle.exponent;

        require!(current_time - last_updated <= max_age, Errors::StaleOracle);

        Ok((price, exponent))
    }

    pub fn is_admin(&mut self, admin: Pubkey) -> bool {
        return self.admin_registry.is_admin(admin);
    }
}
