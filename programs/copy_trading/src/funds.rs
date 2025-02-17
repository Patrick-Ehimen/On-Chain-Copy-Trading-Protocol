use anchor_lang::prelude::*;
use anchor_lang::system_program;

declare_id!("BoXJetaQMDEpgBpsqpoiGbKP2gGEDcvUkQPZNdJHqE65");

#[program]
pub mod funds {
    use super::*;

    /// Initializes a UserFunds account for the depositor.
    /// This account is a PDA derived from the user's pubkey.
    /// Accounts expected by this instruction:
    /// - `user_funds`: The account that will hold the user's funds, initialized here.
    /// - `user`: The account (and signer) that will own the UserFunds account.
    pub fn initialize_user_funds(ctx: Context<InitializeUserFunds>) -> Result<()> {
        let user_funds = &mut ctx.accounts.user_funds;
        user_funds.owner = ctx.accounts.user.key();
        user_funds.balance = 0;
        Ok(())
    }

    /// Deposits `amount` lamports from the user into their UserFunds account.
    pub fn deposit_funds(ctx: Context<DepositFunds>, amount: u64) -> Result<()> {
        let user = &mut ctx.accounts.user;
        let user_funds = &mut ctx.accounts.user_funds;

        // Ensure the caller owns this UserFunds account.
        if user.key() != user_funds.owner {
            return Err(ErrorCode::NotAuthorized.into());
        }

        // Check that the user has enough lamports.
        if **user.to_account_info().lamports.borrow() < amount {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        // Transfer lamports from the user's wallet to the UserFunds account.
        **user.to_account_info().try_borrow_mut_lamports()? -= amount;
        **user_funds.to_account_info().try_borrow_mut_lamports()? += amount;

        // Update the stored balance.
        user_funds.balance = user_funds
            .balance
            .checked_add(amount)
            .ok_or(ErrorCode::Overflow)?;
        Ok(())
    }

    /// Withdraws `amount` lamports from the UserFunds account back to the user.
    pub fn withdraw_funds(ctx: Context<WithdrawFunds>, amount: u64) -> Result<()> {
        let user = &mut ctx.accounts.user;
        let user_funds = &mut ctx.accounts.user_funds;

        // Ensure the caller owns this UserFunds account.
        if user.key() != user_funds.owner {
            return Err(ErrorCode::NotAuthorized.into());
        }

        // Check that there are sufficient funds.
        if user_funds.balance < amount {
            return Err(ErrorCode::InsufficientFunds.into());
        }

        // Update the stored balance.
        user_funds.balance = user_funds
            .balance
            .checked_sub(amount)
            .ok_or(ErrorCode::Underflow)?;

        // Transfer lamports from the UserFunds account back to the user's wallet.
        **user_funds.to_account_info().try_borrow_mut_lamports()? -= amount;
        **user.to_account_info().try_borrow_mut_lamports()? += amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeUserFunds<'info> {
    /// The UserFunds account to be initialized.
    /// Initialized with a space for the UserFunds struct and owned by the system program.
    #[account(
        init,
        payer = user,
        space = 8 + 32 + 8,
        seeds = [b"user_funds", user.key().as_ref()],
        bump
    )]
    pub user_funds: Account<'info, UserFunds>,
    /// The user who will pay for the account initialization and will own the UserFunds account.
    #[account(mut)]
    pub user: Signer<'info>,
    /// The system program for account creation.
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositFunds<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    // The UserFunds account must already be initialized.
    #[account(
        mut,
        seeds = [b"user_funds", user.key().as_ref()],
        bump,
        constraint = user_funds.owner == user.key() @ ErrorCode::NotAuthorized
    )]
    pub user_funds: Account<'info, UserFunds>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFunds<'info> {
    /// The user who wants to withdraw funds, must also own the UserFunds account.
    #[account(mut)]
    pub user: Signer<'info>,
    /// The UserFunds account holding the funds to be withdrawn.
    /// - Seeds: `[b"user_funds", user.key().as_ref()]`
    /// - Must be owned by the calling `user`.
    #[account(
        mut,
        seeds = [b"user_funds", user.key().as_ref()],
        bump,
        constraint = user_funds.owner == user.key() @ ErrorCode::NotAuthorized
    )]
    pub user_funds: Account<'info, UserFunds>,
    /// The system program for processing lamport transfers.
    pub system_program: Program<'info, System>,
}

#[account]
pub struct UserFunds {
    /// Owner of the funds, represented by their public key.
    pub owner: Pubkey,
    /// Current balance of the user's funds in lamports.
    pub balance: u64,
}

#[error_code]
pub enum ErrorCode {
    /// Error indicating that there are not enough funds available for the transaction.
    #[msg("Insufficient funds available.")]
    InsufficientFunds,

    /// Error indicating that an arithmetic operation caused an overflow.
    #[msg("Arithmetic overflow occurred.")]
    Overflow,

    /// Error indicating that an arithmetic operation caused an underflow.
    #[msg("Arithmetic underflow occurred.")]
    Underflow,

    /// Error indicating that the user is not authorized to perform the requested action.
    #[msg("User is not authorized for this action.")]
    NotAuthorized,
}
