#! /bin/bash

# used to resync the Anchor.toml file dumping any `test.validator.clone` accounts to disk


RPC_URL="$1"

grep -A 1 "test.validator.clone" Anchor.toml | grep address | awk -F '=' '{print $2}' | tr -d '" ' | tee accounts.txt

while IFS= read -r account; do
	solana --url "$RPC_URL" account "$i" --output-file "deps/accounts/$account.json" --output json
done < accounts.txt

rm accounts.txt
