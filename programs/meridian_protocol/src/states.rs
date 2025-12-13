use anchor_lang::prelude::*;

use crate::errors::Errors;

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███

#[account]
#[derive(InitSpace)]
pub struct LendingPool {
    pub owner: Pubkey, //AUTHORITY/OWNER OF THE LENDING POOL
    pub admin_registry: Pubkey,
    pub total_deposited_usdc: u64,
    pub total_borrowed: u64,

    pub protocol_admin_count: u8,
    pub withdrawal_epoch: i64,       //WITHDRAWAL EPOCH PERIOD
    pub lp_total_supply: u64,        //TOTAL LP SUPPLY/SHARES MINTED
    pub usdc_mint: Pubkey,           //MINT FOR THE USDC
    pub protocol_usdc_vault: Pubkey, //VAULT FOR THE PROTOCOL USDC TREASURY
    pub loan_to_value_bps: u16,

    //BUMP
    pub bump_lending_pool: u8,
    pub bump_seize_vault: u8,
    pub bump_verification_vault: u8,

    //LIQUIDATION
    pub liquidation_threshold_bps: u16,
    pub liquidation_penalty_bps: u16,
    pub liquidator_reward_bps: u16,

    //UTILIZATION RATE TIERS
    pub utilization_rate_tier_1_bps: u16,
    pub utilization_rate_tier_2_bps: u16,
    pub utilization_rate_tier_3_bps: u16,
    pub utilization_rate_tier_4_bps: u16,
    pub utilization_rate_tier_5_bps: u16,

    //APR TIERS
    pub apr_tier_1_bps: u16,
    pub apr_tier_2_bps: u16,
    pub apr_tier_3_bps: u16,
    pub apr_tier_4_bps: u16,
    pub apr_tier_5_bps: u16,

    //FEES
    pub early_withdrawal_fee_bps: u16, //EARLY WITHDRAWAL FEE FOR THE LENDER (5%)..
    pub origination_fee_bps: u16,      //ORIGINATION FEE  (1% for the borrowers)..

    //POOL STATE
    pub is_locked: bool, //IN CASE OF AN EMERGENCY POOL CAN BE LOCKED BY THE ADMIN...

    //PROTOCOL TREASURIES
    pub collateral_escrow: Pubkey,
    pub collateral_verification_escrow: Pubkey,
    pub protocol_fee_vault: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct LoanState {
    pub borrower: Pubkey,
    pub nft_mint: Pubkey,
    pub verification_id: u32,
    pub is_sent_for_verification: bool,
    pub is_verified: bool,
    pub principal_borrowed: u64,
    pub interest_accrued: u64,
    pub outstanding_debt: u64,
    pub borrowed_at: i64,
    pub last_interest_accrued: u64,
    pub collateral_value_usd: u64,
    pub loan_status: u8, //STATUS = 0(Active), 1(REPAID), 2 (LIQUIDATABLE), 3(LIQUIDATED)
    pub bump_borrower_state: u8,
    pub weight_in_grams: u64,
    pub purity_in_bps: u16,

    pub origination_fee: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Lender {
    pub owner: Pubkey,
    pub lp_shares: u64,
    pub deposited_at: i64,
    pub total_deposited: u64,
    pub total_interest_accrued: u64,
    pub bump: u8,
}

#[account]
pub struct AdminRegistry {
    pub admins: Vec<Pubkey>,
}

impl AdminRegistry {
    pub const MAX_ADMINS: usize = 10;

    pub fn space(admin_count: usize) -> usize {
        8 + 4 + (admin_count * 32) //DISCRIMINATOR + VEC LENGTH PREFIX + PUBKEYS
    }

    pub fn add_admin(&mut self, admin: Pubkey) -> Result<()> {
        require!(self.admins.len() <= Self::MAX_ADMINS, Errors::MaxAdmins);
        require!(!self.admins.contains(&admin), Errors::AdminAlreadyExists);
        self.admins.push(admin);
        Ok(())
    }

    pub fn remove_admin(&mut self, admin_index: usize) -> Result<()> {
        self.admins.remove(admin_index);
        Ok(())
    }

    pub fn is_admin(&mut self, admin: Pubkey) -> Result<()> {
        let _ = self.admins.contains(&admin);
        Ok(())
    }
}
