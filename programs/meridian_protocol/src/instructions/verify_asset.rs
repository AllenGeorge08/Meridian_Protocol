use crate::errors::Errors;
use crate::states::{AdminRegistry, LendingPool, LoanState};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[derive(Accounts)]
pub struct Verify_asset<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
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
        associated_token::mint = mint_usdc,
        associated_token::authority = lending_pool.owner,
        associated_token::token_program = token_program,
    )]
    pub lending_pool_usdc_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [b"meridian_pool_admin_registry",lending_pool.key().as_ref()],
        bump = lending_pool.bump_admin_registry
    )]
    pub admin_registry: Box<Account<'info, AdminRegistry>>,
    #[account(
        mut,
        seeds = [b"meridian_borrower_state", borrower_state.borrower.key().as_ref()],
        bump = borrower_state.bump_borrower_state
    )]
    pub borrower_state: Box<Account<'info, LoanState>>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = borrower_state.borrower,
        associated_token::token_program = token_program,
    )]
    pub borrower_usdc_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    ///CHECK: Safe,Will be created on the client side
    #[account(mut)]
    pub rwa_asset: UncheckedAccount<'info>,
    ///CHECK:
    #[account(
        mut,
        seeds = [b"meridian_verification_vault", lending_pool.key().as_ref()],
        bump = lending_pool.bump_verification_vault
    )]
    pub protocol_verification_vault: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    ///CHECK: SAFE
    pub mpl_core_program: AccountInfo<'info>,
}

impl<'info> Verify_asset<'info> {
    pub fn verify_asset(
        &mut self,
        verification_id: u32,
        is_verified: bool,
        purity_in_bps: u16,
        weight_in_grams: i64,
    ) -> Result<()> {
        require!(
            self.admin_registry.is_admin(self.signer.key()),
            Errors::OnlyAdmin
        );
        require!(
            self.borrower_state.verification_id == verification_id,
            Errors::AssetNotVerified
        );
        self.borrower_state.is_verified = is_verified;

        if is_verified {
            self.borrower_state.is_rejected = false;
        } else {
            self.borrower_state.is_rejected = true;
        }

        self.borrower_state.weight_in_grams = weight_in_grams;
        self.borrower_state.purity_in_bps = purity_in_bps;
        self.borrower_state.is_verified = true;
        Ok(())
    }
}
