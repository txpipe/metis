# Tx3 Transaction Documentation

> **⚠️ Work in Progress**
> 
> This documentation is currently under development and subject to change.

## What is the Tx3 File?

The `main.tx3` file contains comprehensive transaction for Partner Chain operations, written in the Tx3. It serves as the complete blueprint for all Partner Chain governance and validator management operations.

### Key Components

The tx3 file defines the following main transaction categories:

- **Governance Operations**: Initialize governance systems, update authorities, and manage multi-signature policies
- **Governed Map Management**: Create, update, and remove key-value storage entries with governance oversight
- **D-Parameter Control**: Set and modify validator candidate limits and system parameters
- **Candidate Management**: Register/deregister validator candidates (both stake pool operators and permissioned validators)
- **Reserve Operations**: Create and manage token reserves, deposits, releases with vesting mechanisms
- **Circulation Supply**: Control illiquid token supply and distribution through mathematical validation

Each transaction includes detailed security requirements, cryptographic validations, input/output specifications, and complete transaction flow documentation. The file serves as both implementation specification and operational guide for Partner Chain deployments.

## Partner Chain Candidate Validator Registration Guide

This guide explains how Cardano Stake Pool Operators (SPOs) can register as validator candidates on a Partner Chain using the transaction defined in the tx3 file.

### Overview

Partner Chains support two types of validators:
- **Registered Candidates**: Stake pool operators who register through cryptographic proof of stake pool ownership
- **Permissioned Candidates**: Pre-approved validators selected by governance (not covered in this guide)

This guide focuses on **registered candidate** registration for SPOs.

### Prerequisites

#### 1. Stake Pool Ownership
- You must own and operate a Cardano stake pool
- Have access to your stake pool's private keys for signing
- Your stake pool should be properly registered on the Cardano mainnet

#### 2. Required Keys and Addresses
You'll need the following cryptographic materials:

##### Cardano Keys:
- **Stake Pool Private Key**: For signing stake ownership proof
- **Stake Pool Public Key**: Your stake pool's public key (from pool.cert)

##### Partner Chain Keys:
- **Partner Chain Private Key**: A new dedicated key for Partner Chain operations
- **Partner Chain Public Key**: Corresponding public key
- **Candidate Keys**: Additional consensus keys for different protocols:
  - **Aura Key**: For block production authority
  - **Grandpa Key**: For block finalization voting
  - Additional protocol-specific keys as required

##### Transaction Keys:
- **Payment Private Key**: For funding transactions and paying fees
- **Candidate Private Key**: For signing registration transactions

#### 3. UTxO Requirements
- **Registration UTxO**: A specific UTxO that serves as both payment and registration proof
- **Source UTxO**: Additional ADA for transaction fees and minimum UTxO requirements
- **Funding**: Sufficient ADA to cover:
  - Transaction fees
  - Minimum UTxO value for registration output

### Registration Process


#### Step 1: Create Cryptographic Proofs

##### Stake Ownership Signature
Create a signature proving you control the stake pool:

```
# Message Format
RegisterValidatorMessage {
  # Genesis UTXO identifying the specific Partner Chain instance
	genesis_utxo,
	# ECDSA public key for the validator on the Partner Chain
	sidechain_pub_key,
	# UTXO consumed in the registration transaction for uniqueness
	registration_utxo,
}
```

You need to sign this message with your SPO and sidechain keys.

#### Step 2: Prepare Registration Data

Collect all required registration parameters:

```typescript
// CandidateRegistration structure
const registrationData = {
  stake_ownership_pub_key: "02a1b2c3d4...",      // Your stake pool public key (hex)
  stake_ownership_signature: "3044022034f...",    // Stake pool signature (hex)
  partner_chain_pub_key: "025f8e9a1b...",        // Partner Chain public key (hex)
  partner_chain_signature: "30450221...",        // Partner Chain signature (hex)
  registration_utxo: "tx_hash#output_index",     // UTxO reference for registration
  own_pkh: "a2b3c4d5e6f7...",                    // Your payment key hash (hex)
  candidate_keys: [                              // Additional consensus keys
    "04aura_key_bytes...",                       // Aura consensus key
    "04grandpa_key_bytes...",                    // Grandpa consensus key
    // ... additional protocol keys
  ]
};
```

#### Step 3: Execute Registration Transaction

Build and submit the registration transaction using `trix invoke` and select `register_candidate`

### Transaction Flow Details

The `register_candidate` transaction performs the following operations:

1. **Consumes Inputs**:
   - Source UTxO for fees from candidate address
   - Registration UTxO as proof of registration rights
   - Any existing registration UTxOs (for updates)

2. **Creates Outputs**:
   - New candidate UTxO at the committee candidate validator address
   - Change output returning remaining ADA to candidate address

3. **Validation**:
   - Verifies stake pool ownership signature
   - Validates Partner Chain signature
   - Ensures proper UTxO consumption and creation

4. **Data Storage**:
   - Stores complete registration data in candidate UTxO datum
   - Includes version information for future upgrades

### Registration Updates

To update an existing registration (e.g., new consensus keys):

1. Identify your existing registration UTxOs
2. Include them in the `existing_registration_utxos` parameter
3. Execute the registration transaction with updated data
4. The transaction will consolidate old registrations into a new one

### Deregistration Process

To remove your candidacy, use `trix invoke` and select `deregister_candidate` transaction:
