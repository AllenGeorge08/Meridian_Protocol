use anchor_lang::prelude::*;

use crate::AdminRegistry;
// use crate::constants::{GOLD_USD_PRICE_FEED, MAX_AGE};
use crate::errors::Errors;
use crate::states::{LendingPool, LoanState, MockOracleState};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};
use mpl_core::instructions::TransferV1CpiBuilder;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[derive(Accounts)]
pub struct UpdateCollateralValuation<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    ///CHECK: It's the borrower's account
    #[account(mut)]
    pub borrower: UncheckedAccount<'info>,
    #[account(mut)]
    pub mint_usdc: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"meridian_pool",lending_pool.owner.as_ref()],
        bump = lending_pool.bump_lending_pool
    )]
    pub lending_pool: Box<Account<'info, LendingPool>>,
    #[account(
        mut,
        seeds = [b"meridian_borrower_state", borrower.key().as_ref()],
        bump = borrower_state.bump_borrower_state
    )]
    pub borrower_state: Box<Account<'info, LoanState>>,
    #[account(
        mut,
        seeds = [b"meridian_pool_admin_registry",lending_pool.key().as_ref()],
        bump = lending_pool.bump_admin_registry
    )]
    pub admin_registry: Box<Account<'info, AdminRegistry>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub price_update: Account<'info, PriceUpdateV2>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    ///CHECK: SAFE
    pub mpl_core_program: AccountInfo<'info>,
}

impl<'info> UpdateCollateralValuation<'info> {
    pub fn update_collateral_valuation(&mut self, amount: u64) -> Result<()> {
        require!(self.admin_registry.is_admin(self.signer.key()), Errors::OnlyAdmin);
        self.borrower_state.collateral_value_usd = amount;
        msg!("Admin Updated Collateral Valuation of : {} user's collateral to: {}", self.borrower.key(),amount);
        Ok(())
    }
}
