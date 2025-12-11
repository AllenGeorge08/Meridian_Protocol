use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info>{
    pub authority: Signer,
    pub admin: Box<Vec<Pubkey>>,
    

}