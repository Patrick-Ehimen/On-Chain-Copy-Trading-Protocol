use anchor_lang::prelude::*;

declare_id!("BoXJetaQMDEpgBpsqpoiGbKP2gGEDcvUkQPZNdJHqE65");

const MAX_TRADERS: usize = 100;

pub mod master_traders {
    use super::*;

    /// Initializes a new trader list with the admin's public key
    /// and an empty vector for traders.
    ///
    /// @param ctx - the context for the Initialize instruction
    /// @return Result<()> indicating success or failure of the operation
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.trader_list.admin = ctx.accounts.admin.key();
        ctx.accounts.trader_list.traders = Vec::new();
        Ok(())
    }

    /// Adds a new trader to the trader list.
    ///
    /// @param ctx - the context for adding a trader,
    /// which includes references to trader list and admin accounts.
    /// @param trader - the public key of the trader to be added to the list.
    /// @return Result<()> indicating success or failure of the operation.
    pub fn add_trader(ctx: Context<AddTrader>, trader: Pubkey) -> Result<()> {
        // Check admin authorization
        if ctx.accounts.admin.key() != ctx.accounts.trader_list.admin {
            return Err(error!(ErrorCode::NotAdmin));
        }

        // Check list capacity
        if ctx.accounts.trader_list.traders.len() >= MAX_TRADERS {
            return Err(error!(ErrorCode::TraderListFull));
        }

        ctx.accounts.trader_list.traders.push(trader);
        Ok(())
    }

    /// Removes a trader from the trader list.
    ///
    /// @param ctx - the context for removing a trader,
    /// which includes references to trader list and admin accounts.
    /// @param trader - the public key of the trader to be removed.
    /// @return Result<()> indicating success or failure of the operation.
    // Check admin authorization
    pub fn remove_trader(ctx: Context<RemoveTrader>, trader: Pubkey) -> Result<()> {
        if ctx.accounts.admin.key() != ctx.accounts.trader_list.admin {
            return Err(error!(ErrorCode::NotAdmin));
        }

        // Find and remove trader
        let index = ctx
            .accounts
            .trader_list
            .traders
            .iter()
            .position(|x| *x == trader)
            .ok_or(error!(ErrorCode::TraderNotFound))?;

        ctx.accounts.trader_list.traders.remove(index);
        Ok(())
    }

    #[error_code]
    pub enum ErrorCode {
        /// Error indicating that the caller is not the admin.
        #[msg("You are not the admin")]
        NotAdmin,
        /// Error indicating that the specified trader was not found in the list.
        #[msg("Trader not found in the list")]
        TraderNotFound,
        /// Error indicating that the trader list is full and cannot accept more traders.
        #[msg("Trader list is full (max 100)")]
        TraderListFull,
    }
}

// Account structure for initialization
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Creates a new trader list account, with admin as the payer, and allocates space
    /// for storing admin's public key, the length of the traders vector, and space for MAX_TRADERS.
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

/// Data structure for storing the list of traders, managed by an admin.
#[account]
pub struct TraderList {
    /// The public key of the admin who manages the trader list.
    pub admin: Pubkey,
    /// A vector containing public keys of all traders.
    pub traders: Vec<Pubkey>,
}

#[derive(Accounts)]
pub struct AddTrader<'info> {
    /// The trader list account to which a new trader will be added.
    /// Must be mutable to allow changes.
    #[account(mut)]
    pub trader_list: Account<'info, TraderList>,
    /// The admin signer that authorizes modifications to the trader list.
    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveTrader<'info> {
    /// The trader list account from which a trader will be removed.
    /// Must be mutable to allow changes.
    #[account(mut)]
    pub trader_list: Account<'info, TraderList>,
    /// The admin signer that authorizes modifications to the trader list.
    pub admin: Signer<'info>,
}
