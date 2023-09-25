#!/bin/sh

ORIGINAL_IDENTITY=$(dfx identity whoami)
dfx identity use t0 2> /dev/null
T0_PRINCIPAL=$(dfx identity get-principal)
dfx identity use t1 2> /dev/null
T1_PRINCIPAL=$(dfx identity get-principal)
dfx identity use t2 2> /dev/null
T2_PRINCIPAL=$(dfx identity get-principal)
dfx identity use $ORIGINAL_IDENTITY 2> /dev/null

dfx deploy --network=$NETWORK nx-gov-main;
NX_GOV_MAIN=$(dfx canister id --network=$NETWORK nx-gov-main)
dfx deploy --network=$NETWORK simple-validator --argument '(principal "'$NX_GOV_MAIN'")'
SIMPLE_VALIDATOR=$(dfx canister id --network=$NETWORK simple-validator)
dfx deploy --network=$NETWORK multisig-voting --argument '(principal "'$NX_GOV_MAIN'", vec {principal "'$T0_PRINCIPAL'"; principal "'$T1_PRINCIPAL'"; principal "'$T2_PRINCIPAL'"})'
MULTISIG_VOTING=$(dfx canister id --network=$NETWORK multisig-voting)
dfx deploy --network=$NETWORK test-canister --argument '(principal "'$NX_GOV_MAIN'")'
TEST_CANISTER=$(dfx canister id --network=$NETWORK test-canister)

dfx canister call --network=$NETWORK nx-gov-main initialize '(
    principal "'$SIMPLE_VALIDATOR'", 
    principal "'$MULTISIG_VOTING'",
    principal "2vxsx-fae", 
    vec{principal "'$TEST_CANISTER'"})'



dfx identity use $ORIGINAL_IDENTITY 2> /dev/null
