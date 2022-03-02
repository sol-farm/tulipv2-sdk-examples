use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use vaults::instructions::{new_withdraw_multi_deposit_optimizer_vault_ix, new_withdraw_deposit_tracking_ix, new_register_deposit_tracking_account_ix, new_issue_shares_ix};


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
        let got_tracking = vaults::accounts::derive_tracking_address(
            ctx.accounts.vault.key,
            ctx.accounts.authority.key,
            ctx.accounts.vault_program.key,
        ).0;
        if ctx.accounts.deposit_tracking_account.key().ne(&got_tracking) {
            msg_panic!("invalid deposit tracking account. got {}, want {}", got_tracking, ctx.accounts.deposit_tracking_account.key());
        }
        let got_pda = vaults::accounts::derive_tracking_pda_address(
            &got_tracking,
            ctx.accounts.vault_program.key,
        ).0;
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
        {
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
        }
        Ok(())
    }
    /// deposits `amount` of the underlying tokens in exchange for a corresponding
    /// amount of shares. these shares are locked within the deposit tracking account
    /// for 15 minutes, after which they can be removed from the deposit tracking account
    /// if desired. generaly speaking this should only be done if you want to
    /// use the tokenized shares elsewhere (ie friktion volts), otherwise
    /// its best to leave them within the deposit tracking account otherwise
    /// so that you can measure your accrued rewards automatically.
    pub fn issue_shares(ctx: Context<IssueShares>, amount: u64) -> Result<()> {
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
        /*
            if this error is returned, it means the depositing_underlying_account
            has less tokens (X) then requested deposit amount (Y)
            Program log: RUNTIME ERROR: a(X) < b(Y)
            Program log: panicked at 'RUNTIME ERROR: a(0) < b(1)', programs/vaults/src/vault_instructions/deposit_tracking/acl_helpers.rs:198:9
        */
        let ix = new_issue_shares_ix(
            ctx.accounts.authority.key(),
            ctx.accounts.vault.key(),
            ctx.accounts.deposit_tracking_account.key(),
            ctx.accounts.deposit_tracking_pda.key(),
            ctx.accounts.vault_pda.key(),
            ctx.accounts.vault_underlying_account.key(),
            ctx.accounts.shares_mint.key(),
            ctx.accounts.receiving_shares_account.key(),
            ctx.accounts.depositing_underlying_account.key(),
            farm_key.into(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.authority.clone(),
                ctx.accounts.vault.clone(),
                ctx.accounts.deposit_tracking_account.clone(),
                ctx.accounts.deposit_tracking_pda.clone(),
                ctx.accounts.vault_pda.clone(),
                ctx.accounts.vault_underlying_account.to_account_info(),
                ctx.accounts.shares_mint.to_account_info(),
                ctx.accounts.receiving_shares_account.to_account_info(),
                ctx.accounts.depositing_underlying_account.to_account_info(),
            ],
        )?;
        Ok(())
    }
    /// withdraws `amount` of shares from the deposit tracking account into the `receiving_shares_account`.
    /// these withdrawn shares still accrue rewards, the rewards accrued are no longer tracked by the deposit
    /// tracking account
    pub fn withdraw_deposit_tracking(ctx: Context<WithdrawDepositTrackingAccount>, amount: u64) -> Result<()> {
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
        let ix = new_withdraw_deposit_tracking_ix(
            ctx.accounts.authority.key(),
            ctx.accounts.deposit_tracking_account.key(),
            ctx.accounts.deposit_tracking_pda.key(),
            ctx.accounts.deposit_tracking_hold_account.key(),
            ctx.accounts.receiving_shares_account.key(),
            ctx.accounts.shares_mint.key(),
            ctx.accounts.vault.key(),
            farm_key.into(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
             &ix,
             &[
                 ctx.accounts.authority.clone(),
                 ctx.accounts.clock.to_account_info(),
                 ctx.accounts.deposit_tracking_account.clone(),
                 ctx.accounts.deposit_tracking_pda.clone(),
                 ctx.accounts.deposit_tracking_hold_account.to_account_info(),
                 ctx.accounts.receiving_shares_account.to_account_info(),
                 ctx.accounts.shares_mint.to_account_info(),
                 ctx.accounts.vault.clone(),
             ],
        )?;
        Ok(())
    }
    /// burns/redeems the `amount` of shares for their corresponding amount
    /// of underlying asset, using the mango standalone vault as the source of funds to withdraw from
    pub fn withdraw_multi_deposit_vault_through_mango(
        ctx: Context<WithdrawMangoMultiDepositOptimizerVault>,
        amount: u64,
    ) -> Result<()> {
        let standalone_vault_accounts = vec![
            AccountMeta::new_readonly(ctx.accounts.mango_group_account.key(), false),
            AccountMeta::new(ctx.accounts.withdraw_vault_mango_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.mango_cache.key(), false),
            AccountMeta::new_readonly(ctx.accounts.mango_root_bank.key(), false),
            AccountMeta::new(ctx.accounts.mango_node_bank.key(), false),
            AccountMeta::new(ctx.accounts.mango_token_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.mango_group_signer.key(), false),
            AccountMeta::new_readonly(ctx.accounts.system_program.key(), false),
        ];
        let ix = new_withdraw_multi_deposit_optimizer_vault_ix(
            ctx.accounts.common_data.authority.key(),
            ctx.accounts.common_data.multi_vault.key(),
            ctx.accounts.common_data.multi_vault_pda.key(),
            ctx.accounts.common_data.withdraw_vault.key(),
            ctx.accounts.common_data.withdraw_vault_pda.key(),
            ctx.accounts.common_data.platform_information.key(),
            ctx.accounts.common_data.platform_config_data.key(),
            ctx.accounts.common_data.lending_program.key(),
            ctx.accounts.common_data.multi_burning_shares_token_account.key(),
            ctx.accounts.common_data.withdraw_burning_shares_token_account.key(),
            ctx.accounts.common_data.receiving_underlying_token_account.key(),
            ctx.accounts.common_data.multi_underlying_withdraw_queue.key(),
            ctx.accounts.common_data.multi_shares_mint.key(),
            ctx.accounts.common_data.withdraw_shares_mint.key(),
            ctx.accounts.common_data.withdraw_vault_underlying_deposit_queue.key(),
            amount,
            standalone_vault_accounts.clone()
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.common_data.authority.clone(),
                ctx.accounts.common_data.multi_vault.clone(),
                ctx.accounts.common_data.multi_vault_pda.clone(),
                ctx.accounts.common_data.withdraw_vault.clone(),
                ctx.accounts.common_data.withdraw_vault_pda.clone(),
                ctx.accounts.common_data.platform_information.clone(),
                ctx.accounts.common_data.platform_config_data.clone(),
                ctx.accounts.common_data.lending_program.clone(),
                ctx.accounts.common_data.multi_burning_shares_token_account.to_account_info(),
                ctx.accounts.common_data.withdraw_burning_shares_token_account.to_account_info(),
                ctx.accounts.common_data.receiving_underlying_token_account.to_account_info(),
                ctx.accounts.common_data.multi_underlying_withdraw_queue.to_account_info(),
                ctx.accounts.common_data.multi_shares_mint.to_account_info(),
                ctx.accounts.common_data.withdraw_shares_mint.to_account_info(),
                ctx.accounts.common_data.withdraw_vault_underlying_deposit_queue.to_account_info(),
                ctx.accounts.mango_group_account.clone(),
                ctx.accounts.withdraw_vault_mango_account.clone(),
                ctx.accounts.mango_cache.clone(),
                ctx.accounts.mango_root_bank.clone(),
                ctx.accounts.mango_node_bank.clone(),
                ctx.accounts.mango_token_account.to_account_info(),
                ctx.accounts.mango_group_signer.clone(),
                ctx.accounts.system_program.to_account_info(),
                ctx.accounts.common_data.clock.to_account_info(),
            ],
        )?;
        Ok(())
    }
    /// burns/redeems the `amount` of shares for their corresponding amount
    /// of underlying asset, using the solend standalone vault as the source of funds to withdraw from
    pub fn withdraw_multi_deposit_vault_through_solend(
        ctx: Context<WithdrawSolendMultiDepositOptimizerVault>,
        amount: u64,
    ) -> Result<()> {
        let standalone_vault_accounts = vec![
            AccountMeta::new_readonly(ctx.accounts.reserve_account.key(), false),
            AccountMeta::new(ctx.accounts.reserve_liquidity_supply.key(), false),
            AccountMeta::new_readonly(ctx.accounts.reserve_collateral_mint.key(), false),
            AccountMeta::new_readonly(ctx.accounts.lending_market_account.key(), false),
            AccountMeta::new(ctx.accounts.derived_lending_market_authority.key(), false),
            AccountMeta::new(ctx.accounts.reserve_pyth_price_account.key(), false),
            AccountMeta::new_readonly(ctx.accounts.reserve_switchboard_price_account.key(), false),
        ];
        let ix = new_withdraw_multi_deposit_optimizer_vault_ix(
            ctx.accounts.common_data.authority.key(),
            ctx.accounts.common_data.multi_vault.key(),
            ctx.accounts.common_data.multi_vault_pda.key(),
            ctx.accounts.common_data.withdraw_vault.key(),
            ctx.accounts.common_data.withdraw_vault_pda.key(),
            ctx.accounts.common_data.platform_information.key(),
            ctx.accounts.common_data.platform_config_data.key(),
            ctx.accounts.common_data.lending_program.key(),
            ctx.accounts.common_data.multi_burning_shares_token_account.key(),
            ctx.accounts.common_data.withdraw_burning_shares_token_account.key(),
            ctx.accounts.common_data.receiving_underlying_token_account.key(),
            ctx.accounts.common_data.multi_underlying_withdraw_queue.key(),
            ctx.accounts.common_data.multi_shares_mint.key(),
            ctx.accounts.common_data.withdraw_shares_mint.key(),
            ctx.accounts.common_data.withdraw_vault_underlying_deposit_queue.key(),
            amount,
            standalone_vault_accounts.clone()
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.common_data.authority.clone(),
                ctx.accounts.common_data.multi_vault.clone(),
                ctx.accounts.common_data.multi_vault_pda.clone(),
                ctx.accounts.common_data.platform_information.clone(),
                ctx.accounts.common_data.platform_config_data.clone(),
                ctx.accounts.common_data.lending_program.clone(),
                ctx.accounts.common_data.multi_burning_shares_token_account.to_account_info(),
                ctx.accounts.common_data.withdraw_burning_shares_token_account.to_account_info(),
                ctx.accounts.common_data.receiving_underlying_token_account.to_account_info(),
                ctx.accounts.common_data.multi_underlying_withdraw_queue.to_account_info(),
                ctx.accounts.common_data.multi_shares_mint.to_account_info(),
                ctx.accounts.common_data.withdraw_shares_mint.to_account_info(),
                ctx.accounts.common_data.withdraw_vault_underlying_deposit_queue.to_account_info(),
                ctx.accounts.reserve_account.clone(),
                ctx.accounts.reserve_liquidity_supply.to_account_info(),
                ctx.accounts.reserve_collateral_mint.to_account_info(),
                ctx.accounts.lending_market_account.clone(),
                ctx.accounts.derived_lending_market_authority.clone(),
                ctx.accounts.reserve_pyth_price_account.to_account_info(),
                ctx.accounts.reserve_switchboard_price_account.clone(),
            ],
        )?;
        Ok(())
    }
    /// burns/redeems the `amount` of shares for their corresponding amount
    /// of underlying asset, using the tulip standalone vault as the source of funds to withdraw from
    pub fn withdraw_multi_deposit_vault_through_tulip(
        ctx: Context<WithdrawTulipMultiDepositOptimizerVault>,
        amount: u64,
    ) -> Result<()> {
        let standalone_vault_accounts = vec![
            AccountMeta::new_readonly(ctx.accounts.reserve_account.key(), false),
            AccountMeta::new(ctx.accounts.reserve_liquidity_supply.key(), false),
            AccountMeta::new_readonly(ctx.accounts.reserve_collateral_mint.key(), false),
            AccountMeta::new_readonly(ctx.accounts.lending_market_account.key(), false),
            AccountMeta::new(ctx.accounts.derived_lending_market_authority.key(), false),
            AccountMeta::new(ctx.accounts.reserve_pyth_price_account.key(), false),
        ];
        let ix = new_withdraw_multi_deposit_optimizer_vault_ix(
            ctx.accounts.common_data.authority.key(),
            ctx.accounts.common_data.multi_vault.key(),
            ctx.accounts.common_data.multi_vault_pda.key(),
            ctx.accounts.common_data.withdraw_vault.key(),
            ctx.accounts.common_data.withdraw_vault_pda.key(),
            ctx.accounts.common_data.platform_information.key(),
            ctx.accounts.common_data.platform_config_data.key(),
            ctx.accounts.common_data.lending_program.key(),
            ctx.accounts.common_data.multi_burning_shares_token_account.key(),
            ctx.accounts.common_data.withdraw_burning_shares_token_account.key(),
            ctx.accounts.common_data.receiving_underlying_token_account.key(),
            ctx.accounts.common_data.multi_underlying_withdraw_queue.key(),
            ctx.accounts.common_data.multi_shares_mint.key(),
            ctx.accounts.common_data.withdraw_shares_mint.key(),
            ctx.accounts.common_data.withdraw_vault_underlying_deposit_queue.key(),
            amount,
            standalone_vault_accounts.clone()
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.common_data.authority.clone(),
                ctx.accounts.common_data.multi_vault.clone(),
                ctx.accounts.common_data.multi_vault_pda.clone(),
                ctx.accounts.common_data.platform_information.clone(),
                ctx.accounts.common_data.platform_config_data.clone(),
                ctx.accounts.common_data.lending_program.clone(),
                ctx.accounts.common_data.multi_burning_shares_token_account.to_account_info(),
                ctx.accounts.common_data.withdraw_burning_shares_token_account.to_account_info(),
                ctx.accounts.common_data.receiving_underlying_token_account.to_account_info(),
                ctx.accounts.common_data.multi_underlying_withdraw_queue.to_account_info(),
                ctx.accounts.common_data.multi_shares_mint.to_account_info(),
                ctx.accounts.common_data.withdraw_shares_mint.to_account_info(),
                ctx.accounts.common_data.withdraw_vault_underlying_deposit_queue.to_account_info(),
                ctx.accounts.reserve_account.clone(),
                ctx.accounts.reserve_liquidity_supply.to_account_info(),
                ctx.accounts.reserve_collateral_mint.to_account_info(),
                ctx.accounts.lending_market_account.clone(),
                ctx.accounts.derived_lending_market_authority.clone(),
                ctx.accounts.reserve_pyth_price_account.to_account_info(),
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


#[derive(Accounts)]
pub struct IssueShares<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_account: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_pda: AccountInfo<'info>,
    pub vault_pda: AccountInfo<'info>,
    #[account(mut)]
    pub shares_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    /// the account which will receive the issued shares
    /// this is the deposit_tracking_hold_account
    pub receiving_shares_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// the account owned by the authority which contains the underlying tokens
    /// we want to deposit in exchange for the vault shares
    pub depositing_underlying_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// the underlying token account that is owned by the vault pda
    /// which holds the underlying tokens until they are swept into the farm.
    ///
    /// also known as the deposit queue account
    pub vault_underlying_account: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub vault_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct WithdrawDepositTrackingAccount<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_account: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_pda: AccountInfo<'info>,
    #[account(mut)]
    pub deposit_tracking_hold_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// this is the token account owned by the authority that
    /// should receive the tokenized shares which are being removed
    /// from the deposit tracking account. do note that this means
    /// these shares are no longer being tracked by the deposit tracking
    /// account, and any newly accrued rewards tracked by the deposit tracking
    /// account will reflect the remaining balance that hasn't been withdrawn
    /// 
    /// **the shares that are being withdrawn still accrue rewards the same as shares that are held by the deposit tracking account**
    pub receiving_shares_account: Box<Account<'info, TokenAccount>>,
    pub shares_mint: AccountInfo<'info>,
    #[account(mut)]
    pub vault: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    pub vault_program: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct WithdrawMultiDepositOptimizerVault<'info> {
    #[account(signer)]
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub multi_vault: AccountInfo<'info>,
    pub multi_vault_pda: AccountInfo<'info>,
    #[account(mut)]
    pub withdraw_vault: AccountInfo<'info>,
    pub withdraw_vault_pda: AccountInfo<'info>,
    pub platform_information: AccountInfo<'info>,
    pub platform_config_data: AccountInfo<'info>,
    #[account(mut)]
    /// this is the token account owned by the authority for the multi vault
    /// shares mint, which are the tokens we are burning/redeeming in exchange
    /// for the underlying asset
    pub multi_burning_shares_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// this is the account owned by the multi vault pda that holds the tokenized
    /// shares issued by the withdraw vault. 
    pub withdraw_burning_shares_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// this is the account owned by the authority which will receive the underlying
    pub receiving_underlying_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    /// this is the underlying token account owned by the multi deposit vault
    /// which is used to temporarily store tokens during the token withdraw process
    pub multi_underlying_withdraw_queue: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub multi_shares_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub withdraw_shares_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    /// this is the underlying token account owned by the withdraw vault we are
    /// removing underlying assets from
    pub withdraw_vault_underlying_deposit_queue: Box<Account<'info, TokenAccount>>,
    pub clock: Sysvar<'info, Clock>,
    pub token_program: AccountInfo<'info>,
    pub lending_program: AccountInfo<'info>,
    pub vault_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct WithdrawMangoMultiDepositOptimizerVault<'info> {
    /// configuration data common to all multi deposit withdraw instructions
    /// regardless of the underlying vault htey are withdrawing from
    pub common_data: WithdrawMultiDepositOptimizerVault<'info>,
    pub mango_group_account: AccountInfo<'info>,
    #[account(mut)]
    pub withdraw_vault_mango_account: AccountInfo<'info>,
    pub mango_cache: AccountInfo<'info>,
    pub mango_root_bank: AccountInfo<'info>,
    #[account(mut)]
    pub mango_node_bank: AccountInfo<'info>,
    #[account(mut)]
    pub mango_token_account: Box<Account<'info, TokenAccount>>,
    pub mango_group_signer: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawSolendMultiDepositOptimizerVault<'info> {
    /// configuration data common to all multi deposit withdraw instructions
    /// regardless of the underlying vault htey are withdrawing from
    pub common_data: WithdrawMultiDepositOptimizerVault<'info>,
    #[account(mut)]
    pub reserve_account: AccountInfo<'info>,
    #[account(mut)]
    pub reserve_liquidity_supply: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub reserve_collateral_mint: Box<Account<'info, Mint>>,
    pub lending_market_account: AccountInfo<'info>,
    pub derived_lending_market_authority: AccountInfo<'info>,
    pub reserve_pyth_price_account: AccountInfo<'info>,
    pub reserve_switchboard_price_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct WithdrawTulipMultiDepositOptimizerVault<'info> {
    /// configuration data common to all multi deposit withdraw instructions
    /// regardless of the underlying vault htey are withdrawing from
    pub common_data: WithdrawMultiDepositOptimizerVault<'info>,
    #[account(mut)]
    pub reserve_account: AccountInfo<'info>,
    #[account(mut)]
    pub reserve_liquidity_supply: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub reserve_collateral_mint: Box<Account<'info, Mint>>,
    pub lending_market_account: AccountInfo<'info>,
    pub derived_lending_market_authority: AccountInfo<'info>,
    pub reserve_pyth_price_account: AccountInfo<'info>,
}