#!/bin/bash -v

NETWORK=${NETWORK:-local}
OPTS="--network=$NETWORK"
for i in `seq 3 -1 0` ; do echo -ne "\rAbout to run with $OPTS, CTRL-C now if it is not what your want.. ($i) " ; sleep 1 ; done

ORIGINAL_IDENTITY=$(dfx identity whoami)
dfx identity use t0 2> /dev/null
T0_PRINCIPAL=$(dfx identity get-principal)
dfx identity use t1 2> /dev/null
T1_PRINCIPAL=$(dfx identity get-principal)
dfx identity use t2 2> /dev/null
T2_PRINCIPAL=$(dfx identity get-principal)
dfx identity use $ORIGINAL_IDENTITY 2> /dev/null

TEST_CANISTER=$(dfx canister id $OPTS test-canister)
TEST_MESSAGE=$(didc encode '(69,)' -t '(nat64,)')

echo "\e[34m📝 Adding proposal\e[0m"
PROP_ID_0=$(dfx canister call $OPTS nx-gov-main submit '(
    record {memo=vec{};name="Test proposal 0";description="aaa"},
    record {messages=vec{record{
        canister_id=principal "'$TEST_CANISTER'";method="increment";payment=0;
        message=vec{}
        }};depends_on=vec{}},
    variant {At=0},
    variant {In=100000000000000000},
    false,
    )' --identity anonymous | sed -n 's/.*Ok = \([0-9]*\) :.*/\1/p')
PROP_ID=$(dfx canister call $OPTS nx-gov-main submit '(
    record {memo=vec{};name="Test proposal 0";description="aaa"},
    record {messages=vec{record{
        canister_id=principal "'$TEST_CANISTER'";method="increment";payment=0;
        message=vec{}
        }};depends_on=vec{'$PROP_ID_0'}},
    variant {At=0},
    variant {In=100000000000000000},
    false,
    )' --identity anonymous | sed -n 's/.*Ok = \([0-9]*\) :.*/\1/p')

echo "\e[34m🔍 Proposal status:\e[0m"
dfx canister call $OPTS nx-gov-main get_proposal_states '('$PROP_ID',1)'
echo "\e[34m🔄 Validating proposal:\e[0m"
dfx canister call $OPTS simple-validator sync_with_governance
echo "\e[34m🔍 Proposal status:\e[0m"
dfx canister call $OPTS nx-gov-main get_proposal_states '('$PROP_ID',1)'

echo "\e[34m🔄 Syncing with voting canister:\e[0m"
dfx canister call $OPTS multisig-voting sync_with_governance
echo "\e[34m🗳️ Voting:\e[0m"
dfx identity use t0
dfx canister call $OPTS multisig-voting vote_proposal '('$PROP_ID',variant{Yes})'
dfx identity use t1
dfx canister call $OPTS multisig-voting vote_proposal '('$PROP_ID',variant{No})'
dfx identity use t2
dfx canister call $OPTS multisig-voting vote_proposal '('$PROP_ID',variant{Yes})'
echo "\e[34m🔍 Check voting status:\e[0m"
dfx canister call $OPTS multisig-voting get_proposal_state '('$PROP_ID')'
echo "\e[34m📤 Submit vote result to governance:\e[0m"
dfx canister call $OPTS multisig-voting submit_vote_result '('$PROP_ID')'
echo "\e[34m🔍 Proposal status:\e[0m"
dfx canister call $OPTS nx-gov-main get_proposal_states '('$PROP_ID',1)'

echo "\e[34m🚀 Executing proposal:\e[0m"
dfx canister call $OPTS nx-gov-main execute '('$PROP_ID')' --identity anonymous
echo "\e[34m🔍 Proposal status:\e[0m"
dfx canister call $OPTS nx-gov-main get_proposal_states '('$PROP_ID',1)'

