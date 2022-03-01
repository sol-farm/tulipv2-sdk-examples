use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use vaults::instructions::new_register_deposit_tracking_account_ix;


declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod examples {
    use common::msg_panic;

    use super::*;
    pub fn register_deposit_tracking_account(
        ctx: Context<RegisterDepositTrackingAccount>,
    ) -> Result<()> {
        let farm_key = {
            let loader: AccountLoader<vaults::accounts::multi_optimizer::MultiDepositOptimizerV1> = AccountLoader::try_from_unchecked(
                &ctx.accounts.vault_program.key(),
                &ctx.accounts.vault,
            )?;
            {
                let vault = loader.load()?;
                vault.base.farm
            }
        };
        let vault_key = ctx.accounts.vault.key();
        let authority_key = ctx.accounts.authority.key();
        let (got_tracking, tracking_nonce) = vaults::accounts::derive_tracking_address(
            ctx.accounts.vault.key,
            ctx.accounts.authority.key,
            ctx.accounts.vault_program.key,
        );
        if ctx.accounts.deposit_tracking_account.key().ne(&got_tracking) {
            msg_panic!("invalid deposit tracking account. got {}, want {}", got_tracking, ctx.accounts.deposit_tracking_account.key());
        }
        let (got_pda, pda_nonce) = vaults::accounts::derive_tracking_pda_address(
            &got_tracking,
            ctx.accounts.vault_program.key,
        );
        if ctx.accounts.deposit_tracking_pda.key().ne(&got_pda) {
            msg_panic!("invalid deposit tracking pda. got {}, want {}", got_pda, ctx.accounts.deposit_tracking_pda.key());
        }
        // create the associate
        {
            let ix = spl_associated_token_account::create_associated_token_account(
                ctx.accounts.authority.key,
                ctx.accounts.deposit_tracking_pda.key,
                &ctx.accounts.shares_mint.key(),
            );
            anchor_lang::solana_program::program::invoke(
                &ix,
                &[
                    ctx.accounts.authority.clone(),
                    ctx.accounts.deposit_tracking_pda.clone(),
                    ctx.accounts.shares_mint.to_account_info(),
                    ctx.accounts.token_program.clone(),
                    ctx.accounts.deposit_tracking_hold_account.clone(),
                    ctx.accounts.rent.to_account_info(),
                ],
            )?;
        }
        let ix = new_register_deposit_tracking_account_ix(
            ctx.accounts.authority.key(),
            ctx.accounts.vault.key(),
            ctx.accounts.deposit_tracking_account.key(),
            ctx.accounts.deposit_tracking_queue_account.key(),
            ctx.accounts.deposit_tracking_hold_account.key(),
            ctx.accounts.shares_mint.key(),
            ctx.accounts.deposit_tracking_pda.key(),
            farm_key.into()
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.authority.clone(),
                ctx.accounts.vault.clone(),
                ctx.accounts.deposit_tracking_account.clone(),
                ctx.accounts.deposit_tracking_queue_account.clone(),
                ctx.accounts.deposit_tracking_hold_account.clone(),
                ctx.accounts.shares_mint.to_account_info(),
                ctx.accounts.deposit_tracking_pda.clone(),
                ctx.accounts.rent.to_account_info(),
            ],
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct RegisterDepositTrackingAccount<'info> {
    #[account(mut, signer)]
    pub authority: AccountInfo<'info>,
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_account: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_queue_account: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_hold_account: AccountInfo<'info>,
    #[account(mut)]
    pub shares_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub underlying_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub deposit_tracking_pda: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub associated_token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub vault_program: AccountInfo<'info>,
}