#!/usr/bin/env bash
set -ex

if [ ! -d "./target/deploy" ]; then
    ./do.sh build
fi

solana_version="1.7.7"

if ! hash solana 2>/dev/null; then
    echo Installing Solana tool suite ...
    sh -c "$(curl -sSfL https://release.solana.com/v${solana_version}/install)"
    export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
fi

keypair="$HOME"/.config/solana/id.json
if [ ! -f "$keypair" ]; then
    echo Generating keypair ...
    solana-keygen new -o "$keypair" --no-passphrase --silent
fi

CLUSTER_URL=""
if [[ $1 == "localnet" ]]; then
    CLUSTER_URL="http://localhost:8899"
elif [[ $1 == "devnet" ]]; then
    CLUSTER_URL="https://devnet.solana.com"
elif [[ $1 == "testnet" ]]; then
    CLUSTER_URL="https://testnet.solana.com"
else
    echo "Unsupported network: $1"
    exit 1
fi

solana config set --url $CLUSTER_URL
sleep 1
solana airdrop 10
STAKING_PROGRAM_ID="$(solana deploy target/deploy/solana_staking_poc.so | awk '/Program Id/ {print $3}')"
jq -n --arg CLUSTER_URL ${CLUSTER_URL} --arg STAKING_PROGRAM_ID ${STAKING_PROGRAM_ID} \
    '{clusterUrl: $CLUSTER_URL, "stakingProgramId": $STAKING_PROGRAM_ID}' > last-deploy.json
