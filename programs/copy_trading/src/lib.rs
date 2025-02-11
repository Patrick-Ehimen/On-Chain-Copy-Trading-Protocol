use anchor_lang::prelude::*;

declare_id!("8sxRmfq8W6gaH9FVH1H5oZzm6S9GE6hRkerhT6jQQvp5");

#[program]
pub mod copy_trading {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
