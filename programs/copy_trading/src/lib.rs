use anchor_lang::prelude::*;

declare_id!("BoXJetaQMDEpgBpsqpoiGbKP2gGEDcvUkQPZNdJHqE65");

const MAX_TRADERS: usize = 100;

pub mod master_traders {
    use super::*;
    
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.trader_list.admin = ctx.accounts.admin.key();
        ctx.accounts.trader_list.traders = Vec::new();
        Ok(())
    }
}

// Account structure for initialization
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init, 
        payer = admin, 
        space = 8 + 32 + 4 + (32 * MAX_TRADERS)
    )]
    pub trader_list: Account<'info, TraderList>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

// Data structure for storing traders
#[account]
pub struct TraderList {
    pub admin: Pubkey,
    pub traders: Vec<Pubkey>,
}