use crate::states::{AdminRegistry, LendingPool};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub mint: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut)]
    pub mint_lp: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        space = 8  + LendingPool::INIT_SPACE,
        payer = authority,
        seeds = [b"meridian_pool",authority.key().as_ref()],
        bump
    )]
    pub lending_pool: Box<Account<'info, LendingPool>>,
    #[account(
        init_if_needed,
        space = 8  + AdminRegistry::space(5),
        payer = authority,
        seeds = [b"meridian_pool_admin_registry",lending_pool.key().as_ref()],
        bump
    )]
    pub admin_registry: Box<Account<'info, AdminRegistry>>,
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = lending_pool,
        associated_token::token_program = token_program,
    )]
    pub lending_pool_usdc_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint_lp,
        associated_token::authority = lending_pool,
        associated_token::token_program = token_program,
    )]
    pub lending_pool_lp_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = mint,
        associated_token::authority = lending_pool,
        associated_token::token_program = token_program,
    )]
    pub protocol_fee_vault: Box<InterfaceAccount<'info, TokenAccount>>,
    ///CHECK: Protocol PDA where the liquidation seized collateral rwa will be sent
    #[account(
        mut,
        seeds = [b"meridian_seize_vault", lending_pool.key().as_ref()],
        bump,
    )]
    pub protocol_seize_vault: UncheckedAccount<'info>,
    ///CHECK: Protocol PDA where the collateral rwa will be sent for verification at the time for borrowing
    #[account(
        mut,
        seeds = [b"meridian_verification_vault", lending_pool.key().as_ref()],
        bump,
    )]
    pub protocol_verification_vault: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize_pool(
        &mut self,
        loan_to_value_bps: u16,
        utilization_rate_tier_1_bps: u16,
        utilization_rate_tier_2_bps: u16,
        utilization_rate_tier_3_bps: u16,
        utilization_rate_tier_4_bps: u16,
        utilization_rate_tier_5_bps: u16,
        apr_tier_1_bps: u16,
        apr_tier_2_bps: u16,
        apr_tier_3_bps: u16,
        apr_tier_4_bps: u16,
        apr_tier_5_bps: u16,
        early_withdrawal_fee_bps: u16,
        origination_fee_bps: u16,
        withdrawal_epoch: i64,
        liquidation_threshold_bps: u16,
        liquidation_penalty_bps: u16,
        liquidator_reward_bps: u16,
        bumps: &InitializeBumps,
    ) -> Result<()> {
        let lending_pool = &mut self.lending_pool;

        lending_pool.owner = self.authority.key();
        lending_pool.admin_registry = self.admin_registry.key();
        lending_pool.withdrawal_epoch = withdrawal_epoch;
        lending_pool.protocol_admin_count = 0;
        lending_pool.usdc_mint = self.mint.key();
        lending_pool.protocol_usdc_vault = self.lending_pool_usdc_ata.key();

        lending_pool.bump_lending_pool = bumps.lending_pool;
        lending_pool.bump_seize_vault = bumps.protocol_seize_vault;
        lending_pool.bump_verification_vault = bumps.protocol_verification_vault;

        lending_pool.liquidation_threshold_bps = liquidation_threshold_bps;
        lending_pool.liquidation_penalty_bps = liquidation_penalty_bps;
        lending_pool.liquidator_reward_bps = liquidator_reward_bps;

        lending_pool.early_withdrawal_fee_bps = early_withdrawal_fee_bps;
        lending_pool.origination_fee_bps = origination_fee_bps;
        lending_pool.loan_to_value_bps = loan_to_value_bps;

        //UTILIZATION RATE TIERS...
        lending_pool.utilization_rate_tier_1_bps = utilization_rate_tier_1_bps;
        lending_pool.utilization_rate_tier_2_bps = utilization_rate_tier_2_bps;
        lending_pool.utilization_rate_tier_3_bps = utilization_rate_tier_3_bps;
        lending_pool.utilization_rate_tier_4_bps = utilization_rate_tier_4_bps;
        lending_pool.utilization_rate_tier_5_bps = utilization_rate_tier_5_bps;

        //APR TIERS
        lending_pool.apr_tier_1_bps = apr_tier_1_bps;
        lending_pool.apr_tier_2_bps = apr_tier_2_bps;
        lending_pool.apr_tier_3_bps = apr_tier_3_bps;
        lending_pool.apr_tier_4_bps = apr_tier_4_bps;
        lending_pool.apr_tier_5_bps = apr_tier_5_bps;

        lending_pool.is_locked = false;

        lending_pool.protocol_fee_vault = self.protocol_fee_vault.key();

        self.log_state();

        Ok(())
    }

    pub fn log_state(&mut self) {
        println!("Lending Pool State: ...");
        println!(
            "Owner of the lending pool is: {}",
            self.lending_pool.owner.key()
        );
        println!(
            " Address of the admin Registry of the pool is : {}",
            self.admin_registry.key()
        );
        println!(
            "Mint of the usdc is : {} ",
            self.lending_pool_usdc_ata.key()
        );
        println!(
            "Address of the protocol's usdc ata is: {}",
            self.lending_pool.protocol_usdc_vault
        );
        println!(
            "Address of the protocol's fee vault is: {}",
            self.lending_pool.protocol_fee_vault
        );
        println!(
            "Liquidation threshold currently is at: {} bps",
            self.lending_pool.liquidation_threshold_bps
        );

        println!(
            "Liquidator rewards(bps) currently is at: {}",
            self.lending_pool.liquidator_reward_bps
        );
        println!(
            "Liquidator penalty(bps) currently is at: {}",
            self.lending_pool.liquidation_penalty_bps
        );
        println!(
            "Current Loan To Value(bps) currently is at: {}",
            self.lending_pool.loan_to_value_bps
        );
        println!("Pool is locked: {}", self.lending_pool.is_locked);
    }
}
