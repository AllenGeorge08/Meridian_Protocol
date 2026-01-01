use crate::errors::Errors;
use crate::states::{AdminRegistry, Lender, LendingPool};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{
    burn, transfer_checked, Burn, Mint, TokenAccount, TokenInterface, TransferChecked,
};

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[derive(Accounts)]
pub struct Withdraw<'info> {
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
        associated_token::authority = lending_pool,
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
        associated_token::authority = lending_pool,
        associated_token::token_program = token_program,
    )]
    pub lending_pool_lp_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = lender,
        associated_token::token_program = token_program,
    )]
    pub lender_usdc_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = lender,
        associated_token::token_program = token_program,
    )]
    pub lender_lp_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw<'info> {
    pub fn withdraw_liquidity(&mut self) -> Result<()> {
        require!(
            self.lender.key() == self.lender_state.owner,
            Errors::InvalidUser
        );
        require!(self.lending_pool.is_locked == false, Errors::PoolLocked);

        //BURN SHARES..
        let burn_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.lender_lp_ata.to_account_info(),
            authority: self.lender.to_account_info(),
        };

        let cpi_program = self.token_program.to_account_info();

        let shares_to_burn = self.lender_state.lp_shares;

        let cpi_ctx = CpiContext::new(cpi_program, burn_accounts);

        burn(cpi_ctx, shares_to_burn)?;

        msg!("Shares burnt: {}", shares_to_burn);

        //WITHDRAWING AMOUNT
        let amount_to_withdraw: u64;
        let current_time = Clock::get()?.unix_timestamp;

        let withdrawable_amount = self.get_total_withdrawable_amount(self.lender_state.lp_shares);

        if self.is_withdrawal_epoch_over(self.lender_state.deposited_at, current_time) {
            amount_to_withdraw = withdrawable_amount;
        } else {
            amount_to_withdraw = self.calculate_early_withdrawal_amount(withdrawable_amount);
        }

        let accounts = TransferChecked {
            from: self.lending_pool_usdc_ata.to_account_info(),
            to: self.lender_usdc_ata.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.lending_pool.to_account_info(),
        };

        let lending_pool_owner = self.lending_pool.owner;
        let seeds = &[
            &b"meridian_pool"[..],
            lending_pool_owner.as_ref(),
            &[self.lending_pool.bump_lending_pool],
        ];

        let signer_seeds = &[&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            accounts,
            signer_seeds,
        );
        transfer_checked(cpi_ctx, amount_to_withdraw, self.mint.decimals)?;

        msg!("Liquidity Amount Withdrawn By: {}", self.lender.key());

        Ok(())
    }

    pub fn is_withdrawal_epoch_over(&mut self, deposited_at: i64, current_time: i64) -> bool {
        let withdrawal_epoch = self.lending_pool.withdrawal_epoch;
        //Is it correct...?
        if current_time - deposited_at >= withdrawal_epoch {
            return true;
        } else {
            return false;
        }
    }

    pub fn calculate_shares_to_mint(&mut self, deposit_amount: u64) -> u64 {
        let lp_supply = self.lending_pool.lp_total_supply;
        let total_liquidity_in_pool = self.lending_pool.total_deposited_usdc;

        if lp_supply == 0 {
            return deposit_amount;
        };

        let shares_to_mint = deposit_amount * lp_supply / total_liquidity_in_pool;
        return shares_to_mint;
    }

    pub fn get_total_withdrawable_amount(&mut self, lp_shares: u64) -> u64 {
        let total_liquidity = self.lending_pool.total_deposited_usdc;
        let total_lp_supply = self.lending_pool.lp_total_supply;
        let lp_share_price = total_liquidity / total_lp_supply as u64;

        let withdrawable_amount = lp_shares * lp_share_price;
        return withdrawable_amount;
    }

    pub fn get_total_interest_earned(
        &mut self,
        total_deposited_collateral: u64,
        total_lp_shares_owned: u64,
    ) -> u64 {
        let total_withdrawable_amount = self.get_total_withdrawable_amount(total_lp_shares_owned);
        let interest_earned = total_withdrawable_amount - total_deposited_collateral;
        return interest_earned;
    }

    pub fn calculate_early_withdrawal_amount(&mut self, total_withdrawal_amount: u64) -> u64 {
        return total_withdrawal_amount * self.lending_pool.early_withdrawal_fee_bps as u64 / 10000;
    }
}
