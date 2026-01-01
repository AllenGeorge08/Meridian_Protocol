use crate::errors::Errors;
use crate::states::{Lender, LendingPool};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{
    mint_to, transfer_checked, Mint, MintTo, TokenAccount, TokenInterface, TransferChecked,
};

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[derive(Accounts)]
pub struct Lending<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub lender: Signer<'info>,
    #[account(mut)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub mint_lp: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"meridian_pool",lending_pool.owner.as_ref()],
        bump = lending_pool.bump_lending_pool
    )]
    pub lending_pool: Box<Account<'info, LendingPool>>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = lending_pool.owner,
        associated_token::token_program = token_program,
    )]
    pub lending_pool_usdc_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = lender,
        space = 8 + Lender::INIT_SPACE,
        seeds = [b"lender_seed", lender.key().as_ref()],
        bump
    )]
    pub lender_state: Box<Account<'info, Lender>>,
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = lending_pool.owner,
        associated_token::token_program = token_program,
    )]
    pub lending_pool_lp_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = lender,
        associated_token::mint = mint,
        associated_token::authority = lender,
        associated_token::token_program = token_program,
    )]
    pub lender_usdc_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = lender,
        associated_token::mint = mint_lp,
        associated_token::authority = lender,
        associated_token::token_program = token_program,
    )]
    pub lender_lp_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Lending<'info> {
    pub fn deposit_liquidity(&mut self, amount_to_deposit: u64) -> Result<()> {
        require!(amount_to_deposit.clone() > 0, Errors::NullDepositNotAllowed);
        require!(!self.lending_pool.is_locked, Errors::PoolLocked);

        let accounts = TransferChecked {
            from: self.lender_usdc_ata.to_account_info(),
            to: self.lending_pool_usdc_ata.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.lender.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();
        let cpi_context = CpiContext::new(cpi_program, accounts);

        transfer_checked(cpi_context, amount_to_deposit, self.mint.decimals)?;

        self.lender_state.deposited_at = Clock::get()?.unix_timestamp;
        self.lender_state.total_deposited = amount_to_deposit;
        self.lender_state.owner = self.lender.key();

        msg!("Amount Deposited: {}", amount_to_deposit);

        self.mint_shares(amount_to_deposit)?;

        Ok(())
    }

    fn mint_shares(&mut self, amount_deposited: u64) -> Result<()> {
        let amount_shares_to_mint = self.calculate_shares_to_mint(amount_deposited);

        let authority = self.lending_pool.owner;

        let accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            authority: self.authority.to_account_info(),
            to: self.lender_lp_ata.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();

        // let lending_pool_owner = self.lending_pool.owner.key();
        // let seeds = &[
        //     b"meridian_pool",
        //     lending_pool_owner.as_ref(),
        //     &[self.lending_pool.bump_lending_pool],
        // ];

        // let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new(cpi_program, accounts);

        mint_to(cpi_ctx, amount_shares_to_mint)?;
        msg!("Minted Lp Tokens to: {}", self.lender_lp_ata.key());

        self.lender_state.lp_shares += amount_shares_to_mint;
        self.lending_pool.lp_total_supply += amount_shares_to_mint;

        Ok(())
    }

    pub fn calculate_shares_to_mint(&mut self, deposit_amount: u64) -> u64 {
        let lp_supply = self.lending_pool.lp_total_supply;
        let total_liquidity_in_pool = self.lending_pool.total_deposited_usdc;

        if lp_supply == 0 {
            return deposit_amount;
        };

        let shares_to_mint = deposit_amount
            .checked_mul(lp_supply)
            .unwrap()
            .checked_div(total_liquidity_in_pool)
            .unwrap();
        return shares_to_mint;
    }
}
