# OpenRank TEE Protocol

A decentralized reputation computation protocol leveraging Trusted Execution Environments (TEEs) and EigenLayer for secure, verifiable PageRank-style calculations.

## Overview

OpenRank TEE is a decentralized protocol that enables secure computation of reputation scores using Trusted Execution Environments (TEEs). The protocol integrates with EigenLayer's restaking infrastructure to provide economic security and operator management, while ensuring computational integrity through TEE attestations and challenge mechanisms.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           OpenRank TEE Protocol                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐          │
│  │     Users       │    │   Challengers   │    │   TEE Nodes     │          │
│  │                 │    │                 │    │   (Computers)   │          │
│  │ - Submit jobs   │    │ - Verify results│    │ - Process jobs  │          │
│  │ - Define domains│    │ - Challenge bad │    │ - Submit results│          │
│  │                 │    │   computations  │    │ - Generate proof│          │
│  └─────────┬───────┘    └─────────┬───────┘    └─────────┬───────┘          │
│            │                      │                      │                  │
│            │                      │                      │                  │
│  ┌─────────▼──────────────────────▼──────────────────────▼───────┐          │
│  │                    Smart Contract Layer                       │          │
│  │                                                               │          │
│  │  ┌─────────────────────────────────────────────────────────┐  │          │
│  │  │            OpenRankManager Contract                     │  │          │
│  │  │                                                         │  │          │
│  │  │ • submitMetaComputeRequest()                            │  │          │
│  │  │ • submitMetaComputeResult()                             │  │          │
│  │  │ • submitMetaChallenge()                                 │  │          │
│  │  │ • Operator allowlisting                                 │  │          │
│  │  │ • Challenge window management                           │  │          │
│  │  └─────────────────────────────────────────────────────────┘  │          │
│  └───────────────────────────────────────────────────────────────┘          │
│                                    │                                        │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────┐          │
│  │                    EigenLayer Integration                     │          │
│  │                                                               │          │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐│          │
│  │  │   Registration  │  │    Slashing     │  │    Rewards      ││          │
│  │  │   Coordinator   │  │   Management    │  │  Distribution   ││          │
│  │  │                 │  │                 │  │                 ││          │
│  │  │ • Operator      │  │ • Economic      │  │ • Performance   ││          │
│  │  │   registration  │  │   security      │  │   incentives    ││          │
│  │  │ • Stake         │  │ • Dispute       │  │ • Token rewards ││          │
│  │  │   management    │  │   resolution    │  │                 ││          │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘│          │
│  └───────────────────────────────────────────────────────────────┘          │
│                                    │                                        │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────┐          │
│  │                 TEE Computation Layer                         │          │
│  │                                                               │          │
│  │  ┌─────────────────────────────────────────────────────────┐  │          │
│  │  │              Trusted Execution Environment              │  │          │
│  │  │                                                         │  │          │
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │  │          │
│  │  │  │   Computer  │  │ Challenger  │  │   Merkle    │      │  │          │
│  │  │  │   Runner    │  │   Runner    │  │    Trees    │      │  │          │
│  │  │  │             │  │             │  │             │      │  │          │
│  │  │  │ • PageRank  │  │ • Result    │  │ • Efficient │      │  │          │
│  │  │  │   algorithm │  │   verify    │  │   proofs    │      │  │          │
│  │  │  │ • Trust     │  │ • Challenge │  │ • Commitment│      │  │          │
│  │  │  │   compute   │  │   generation│  │   schemes   │      │  │          │
│  │  │  └─────────────┘  └─────────────┘  └─────────────┘      │  │          │
│  │  │                                                         │  │          │
│  │  │              Remote Attestation & Proofs                │  │          │
│  │  └─────────────────────────────────────────────────────────┘  │          │
│  └───────────────────────────────────────────────────────────────┘          │
│                                    │                                        │
│                                    ▼                                        │
│  ┌───────────────────────────────────────────────────────────────┐          │
│  │                    Data Storage Layer                         │          │
│  │                                                               │          │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐│          │
│  │  │   Job Metadata  │  │  Trust/Seed     │  │    Results      ││          │
│  │  │                 │  │     Data        │  │   & Proofs      ││          │
│  │  │ • Domain specs  │  │                 │  │                 ││          │
│  │  │ • Algorithm IDs │  │ • Trust graphs  │  │ • Score outputs ││          │
│  │  │ • Parameters    │  │ • Seed vectors  │  │ • Merkle proofs ││          │
│  │  │                 │  │ • Input data    │  │ • Commitments   ││          │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘│          │
│  │                            AWS S3 Storage                     │          │
│  └───────────────────────────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Smart Contract Layer

The **OpenRankManager** contract serves as the coordination layer for the protocol:

- **Compute Requests**: Users submit computation jobs with domain specifications
- **Result Submission**: TEE operators submit computation results with cryptographic commitments
- **Challenge System**: Verification nodes can challenge incorrect results within a time window
- **Operator Management**: Integration with EigenLayer for operator allowlisting and slashing

```solidity
interface IOpenRankManager {
    function submitMetaComputeRequest(bytes32 jobDescriptionId) external returns (uint256);
    function submitMetaComputeResult(uint256 computeId, bytes32 commitment, bytes32 resultsId) external;
    function submitMetaChallenge(uint256 computeId, uint32 subJobId) external;
}
```

### 2. TEE Computation Layer

The core computation happens inside Trusted Execution Environments:

#### Computer Nodes
- Execute OpenRank algorithms (EigenTrust, PageRank variants)
- Process trust graphs and seed vectors
- Generate cryptographic commitments and proofs
- Submit results with TEE attestations

#### Challenger Nodes
- Verify computation results independently
- Challenge suspicious or incorrect computations
- Participate in dispute resolution

#### Key Features
- **Merkle Tree Proofs**: Efficient verification of large datasets
- **Remote Attestation**: TEE-backed proof of execution integrity
- **Parallel Processing**: Optimized computation using Rayon

### 3. EigenLayer Integration

The protocol leverages EigenLayer's restaking infrastructure:

#### Operator Registration
```rust
// Operators must be allowlisted and stake ETH through EigenLayer
function allowlistComputer(address computer) external onlyOwner;
```

#### Economic Security
- Operators stake ETH/LSTs through EigenLayer
- Slashing conditions for malicious behavior
- Rewards for honest computation and verification

#### Middleware Components
- **Registry Coordinator**: Manages operator registration and quorum
- **Slashing Manager**: Handles dispute resolution and penalties
- **Service Manager**: Coordinates between protocol and EigenLayer core

## Protocol Workflow

### 1. Job Submission
```
User                 AWS S3               Smart Contract       TEE Node
 │                     │                       │                 │
 │ Upload job desc     │                       │                 │
 ├────────────────────►│                       │                 │
 │                     │                       │                 │
 │ submitMetaComputeRequest(jobId)             │                 │
 ├────────────────────────────────────────────►│                 │
 │                     │                       │                 │
 │                     │                       │ Emit Event      │
 │                     │                       ├────────────────►│
 │                     │                       │                 │
 │                     │                       │◄────────────────┤
 │                     │                       │   Monitor       │
```

### 2. Computation Phase
```
TEE Node             AWS S3               Smart Contract
   │                   │                       │
   │ Download data     │                       │
   ├──────────────────►│                       │
   │                   │                       │
   │ Execute algorithm │                       │
   ├───────────────────│ (internal)            │
   │                   │                       │
   │ Generate proofs   │                       │
   ├───────────────────│ (internal)            │
   │                   │                       │
   │ Upload results    │                       │
   ├──────────────────►│                       │
   │                   │                       │
   │ submitMetaComputeResult(commitment)       │
   ├──────────────────────────────────────────►│
```

### 3. Challenge Phase
```
Challenger           AWS S3               Smart Contract       EigenLayer
    │                  │                       │                  │
    │ Download results │                       │                  │
    ├─────────────────►│                       │                  │
    │                  │                       │                  │
    │ Verify comp.     │                       │                  │
    ├──────────────────│ (internal)            │                  │
    │                  │                       │                  │
    │ (if incorrect)   │                       │                  │
    │ submitMetaChallenge(computeId)           │                  │
    ├─────────────────────────────────────────►│                  │
    │                  │                       │                  │
    │                  │                       │ Slash operator   │
    │                  │                       ├─────────────────►│
```

## Key Algorithms

### EigenTrust Algorithm
The protocol implements a variant of the EigenTrust algorithm for reputation computation:

```rust
pub fn positive_run(
    trust_map: &HashMap<u64, Vec<(u64, f32)>>,
    seed_map: &HashMap<u64, f32>,
    iterations: usize,
    alpha: f32,
) -> HashMap<u64, f32>
```

Features:
- Iterative trust propagation
- Seed vector incorporation for personalization
- Parallel computation for scalability
- Convergence detection

## Security Model

### TEE Security
- **Hardware-based isolation** ensures computation integrity
- **Remote attestation** proves code execution in genuine TEE
- **Sealed storage** protects sensitive intermediate data

### Economic Security (EigenLayer)
- **Restaked ETH** provides economic guarantees
- **Slashing conditions** penalize malicious operators
- **Challenge mechanism** allows verification without full re-computation

### Cryptographic Security
- **Keccak256** hashing for data integrity
- **Merkle trees** for efficient proof generation
- **Commitment schemes** for result binding

## Data Flow

### Input Data
1. **Trust Graphs**: Weighted directed graphs representing trust relationships
2. **Seed Vectors**: Initial reputation distributions
3. **Algorithm Parameters**: Damping factors, iteration counts, convergence thresholds

### Computation Process
1. **Data Preprocessing**: Normalization and validation
2. **Algorithm Execution**: PageRank/EigenTrust computation
3. **Proof Generation**: Merkle tree construction
4. **Result Packaging**: Commitment and metadata generation

### Output Data
1. **Reputation Scores**: Final computed values
2. **Merkle Proofs**: Verification data
3. **Execution Metadata**: Performance and convergence information

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **EigenLayer**: For the restaking infrastructure and middleware framework
- **OpenRank Protocol**: For the reputation computation algorithms
- **Trusted Execution Environments**: For secure computation guarantees
