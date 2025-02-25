use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("DYFBQCkjzYiQ2rKZBEFW45XpoWbibfLZ5NMGCkyu5wsF");

#[program]
pub mod copy_trading {
    use super::*;

    // Initialize the main copy trading program account
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let copy_trading = &mut ctx.accounts.copy_trading;
        copy_trading.authority = ctx.accounts.authority.key();
        copy_trading.master_trader_count = 0;
        Ok(())
    }

    // Register a new master trader
    pub fn register_master_trader(
        ctx: Context<RegisterMasterTrader>,
        name: String,
        description: String,
    ) -> Result<()> {
        let copy_trading = &mut ctx.accounts.copy_trading;
        let master_trader = &mut ctx.accounts.master_trader;

        // Set master trader account data
        master_trader.authority = ctx.accounts.authority.key();
        master_trader.name = name;
        master_trader.description = description;
        master_trader.total_followers = 0;
        master_trader.total_aum = 0;

        // Increment the master trader counter
        copy_trading.master_trader_count += 1;

        Ok(())
    }

    // Allow a user to follow a trader by depositing funds
    pub fn follow_trader(ctx: Context<FollowTrader>, amount: u64) -> Result<()> {
        let master_trader = &mut ctx.accounts.master_trader;
        let follower = &mut ctx.accounts.follower;

        // Store the master trader public key first before borrowing master_trader as mutable
        let master_trader_key = master_trader.key();

        // Transfer tokens from user to the PDA vault
        let transfer_instruction = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            transfer_instruction,
        );

        token::transfer(cpi_ctx, amount)?;

        // Update follower data
        follower.user = ctx.accounts.user.key();
        follower.master_trader = master_trader_key;
        follower.deposited_amount = amount;
        follower.active = true;

        // Update master trader stats
        master_trader.total_followers += 1;
        master_trader.total_aum += amount;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 // discriminator + pubkey + u64
    )]
    pub copy_trading: Account<'info, CopyTrading>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterMasterTrader<'info> {
    #[account(mut)]
    pub copy_trading: Account<'info, CopyTrading>,

    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 100 + 200 + 8 + 8 // discriminator + pubkey + name + description + followers + aum
    )]
    pub master_trader: Account<'info, MasterTrader>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FollowTrader<'info> {
    #[account(mut)]
    pub master_trader: Account<'info, MasterTrader>,

    #[account(
        init,
        payer = user,
        space = 8 + 32 + 32 + 8 + 1, // discriminator + user + master_trader + amount + active
        seeds = [b"follower", user.key().as_ref(), master_trader.key().as_ref()],
        bump
    )]
    pub follower: Account<'info, Follower>,

    #[account(
        seeds = [b"vault", user.key().as_ref(), master_trader.key().as_ref()],
        bump,
    )]
    /// CHECK: This is a PDA that acts as a vault
    pub vault: AccountInfo<'info>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault_token_account.owner == vault.key(),
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct CopyTrading {
    pub authority: Pubkey,
    pub master_trader_count: u64,
}

#[account]
pub struct MasterTrader {
    pub authority: Pubkey,
    pub name: String,
    pub description: String,
    pub total_followers: u64,
    pub total_aum: u64, // Assets under management
}

#[account]
pub struct Follower {
    pub user: Pubkey,
    pub master_trader: Pubkey,
    pub deposited_amount: u64,
    pub active: bool,
}
