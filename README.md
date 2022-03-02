# Tulip V2 SDK Examples

This repository contains an example program which showcases how to invoke v2 vaults instructions through cpi. 

Note that only the `register_deposit_tracking_account` and `issue_shares` instructions will pass locally, as the remaining instructions require vault sweeping which is both a complex instruciton, and permissioned instruction beyond the scope of this example.

# Usage

Clone the repository locally, and run the following commands (requires anchor 0.22.0)

```shell
$> yarn
$> anchor test
```