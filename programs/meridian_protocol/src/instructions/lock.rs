use anchor_lang::prelude::*;
use crate::{states::*};
use crate::errors::Errors;

// ▄▄▄      ▄▄▄  ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄   ▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄▄▄▄   ▄▄▄▄   ▄▄▄    ▄▄▄ 
// ████▄  ▄████ ███▀▀▀▀▀ ███▀▀███▄  ███  ███▀▀██▄  ███  ▄██▀▀██▄ ████▄  ███ 
// ███▀████▀███ ███▄▄    ███▄▄███▀  ███  ███  ███  ███  ███  ███ ███▀██▄███ 
// ███  ▀▀  ███ ███      ███▀▀██▄   ███  ███  ███  ███  ███▀▀███ ███  ▀████ 
// ███      ███ ▀███████ ███  ▀███ ▄███▄ ██████▀  ▄███▄ ███  ███ ███    ███ 
                                                                         
                                                                                                                                                                 
#[derive(Accounts)]
pub struct LockPool<'info>{
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"meridian_pool",authority.key().as_ref()],
        bump = lending_pool.bump_lending_pool
    )] 
    pub lending_pool: Box<Account<'info,LendingPool>>,
    pub system_program: Program<'info, System>,
}

impl<'info> LockPool<'info>{
    pub fn lock_pool(&mut self) -> Result<()>{
        require!(self.authority.key() == self.lending_pool.owner.key(), Errors::OnlyAuthority);
        require!(!self.lending_pool.is_locked, Errors::PoolAlreadyLocked);
        self.lending_pool.is_locked = true;
        msg!("Pool Locked");
        Ok(())
    }
}