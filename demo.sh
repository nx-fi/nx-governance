#!/bin/bash -v

SLEEP_TIME=3

ORIGINAL_IDENTITY=$(dfx identity whoami)
dfx identity use t0 2> /dev/null
T0_PRINCIPAL=$(dfx identity get-principal)
dfx identity use t1 2> /dev/null
T1_PRINCIPAL=$(dfx identity get-principal)
dfx identity use t2 2> /dev/null
T2_PRINCIPAL=$(dfx identity get-principal)
dfx identity use $ORIGINAL_IDENTITY 2> /dev/null

TEST_CANISTER=$(dfx canister id --network=$NETWORK test-canister)
TEST_MESSAGE=$(didc encode '(69,)' -t '(nat64,)')

echo "\e[95mWelcome to the NX Governance demo!\e[0m \e[94mNX Governance is a cutting-edge multi-chain governance framework built on the Internet Computer.\e[0m"
sleep $SLEEP_TIME
echo "\e[93mIn this demo, we will walk you through the process of adding a proposal, validating it, and then voting on it.\e[0m"
sleep $SLEEP_TIME
echo "\e[92mYou'll see the various statuses a proposal goes through, from its inception to its execution.\e[0m"
sleep $SLEEP_TIME
echo "\e[91mWe'll also showcase some of the checks and balances in place, like trying to vote multiple times or executing a proposal prematurely.\e[0m"
sleep $SLEEP_TIME
echo "\e[96mLet's get started!\e[0m"
sleep $SLEEP_TIME

echo "\e[34m📝 Adding proposal\e[0m"
PROP_ID=$(dfx canister call --network=$NETWORK nx-gov-main submit '(
    record {memo=vec{};name="Test proposal 0";description="aaa"},
    record {messages=vec{record{
        canister_id=principal "'$TEST_CANISTER'";method="increment";payment=0;
        message=vec{}
        }};depends_on=vec{}},
    variant {At=0},
    variant {In=100000000000000000},
    false,
    )' --identity anonymous | sed -n 's/.*Ok = \([0-9]*\) :.*/\1/p')
sleep $SLEEP_TIME
echo "\e[35m🔍 Proposal status:\e[0m"
dfx canister call --network=$NETWORK nx-gov-main get_proposal_states '('$PROP_ID',1)'
sleep $SLEEP_TIME
echo "\e[36m🔎 Validation status in validator:\e[0m"
dfx canister call --network=$NETWORK simple-validator get_proposal_validation '('$PROP_ID')'
sleep $SLEEP_TIME
echo "\e[33m🔄 Validating proposal:\e[0m"
dfx canister call --network=$NETWORK simple-validator sync_with_governance
sleep $SLEEP_TIME
echo "\e[36m🔎 Validation status in validator:\e[0m"
dfx canister call --network=$NETWORK simple-validator get_proposal_validation '('$PROP_ID')'

sleep $SLEEP_TIME
echo "\e[35m🔍 Proposal status:\e[0m"
dfx canister call --network=$NETWORK nx-gov-main get_proposal_states '('$PROP_ID',1)'

sleep $SLEEP_TIME
echo "\e[32m🔄 Syncing with voting canister:\e[0m"
dfx canister call --network=$NETWORK multisig-voting sync_with_governance

sleep $SLEEP_TIME
echo "\e[34m🗳️ Voting on the proposal:\e[0m"
dfx identity use t0
sleep $SLEEP_TIME
echo "\e[32m✅ I'm voting YES!\e[0m"
dfx canister call --network=$NETWORK multisig-voting vote_proposal '('$PROP_ID',variant{Yes})'
sleep $SLEEP_TIME
echo "\e[33m🤔 Let me vote again...\e[0m"
dfx canister call --network=$NETWORK multisig-voting vote_proposal '('$PROP_ID',variant{Yes})'
sleep $SLEEP_TIME
echo "\e[33m...of course that didn't work. Was worth a try though.\e[0m"
sleep $SLEEP_TIME
echo "\e[31m🤔 Let me try to execute the proposal...\e[0m"
dfx canister call --network=$NETWORK multisig-voting submit_vote_result '('$PROP_ID')'
echo "\e[31m⏳ Okay I shall wait for the other votes to come in.\e[0m"
sleep $SLEEP_TIME

dfx identity use anonymous
echo "\e[31m❌ I'm voting No.\e[0m"
dfx canister call --network=$NETWORK multisig-voting vote_proposal '('$PROP_ID',variant{No})'
echo "\e[31m... Oops, I can't vote!\e[0m"
sleep $SLEEP_TIME
dfx identity use t1
echo "\e[33m🤷 I'm voting ABSTAIN.\e[0m"
dfx canister call --network=$NETWORK multisig-voting vote_proposal '('$PROP_ID',variant{Abstain})'
sleep $SLEEP_TIME
dfx identity use t2
echo "\e[32m✅ I'm voting Yes!\e[0m"
dfx canister call --network=$NETWORK multisig-voting vote_proposal '('$PROP_ID',variant{Yes})'
sleep $SLEEP_TIME
dfx identity use t1
echo "\e[35m🔍 Check voting status:\e[0m"
dfx canister call --network=$NETWORK multisig-voting get_proposal_state '('$PROP_ID')'
sleep $SLEEP_TIME

echo "\e[34m📤 Submit vote result to governance:\e[0m"
dfx canister call --network=$NETWORK multisig-voting submit_vote_result '('$PROP_ID')'
sleep $SLEEP_TIME

echo "\e[33m🚀 Executing proposal:\e[0m"
dfx canister call --network=$NETWORK nx-gov-main execute '('$PROP_ID')' --identity anonymous
sleep $SLEEP_TIME

echo "\e[97mThank you for joining us on this journey through the NX Governance demo.\e[0m"

dfx identity use $ORIGINAL_IDENTITY 2> /dev/null
