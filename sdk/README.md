# OpenRank SDK

A command-line SDK for interacting with the OpenRank AVS (Actively Validated Service) - a decentralized trust and reputation system that runs PageRank-style algorithms in Trusted Execution Environments (TEEs).

## What is OpenRank?

OpenRank is a decentralized system for computing trust scores and reputation rankings using the EigenTrust algorithm (a variant of PageRank). It operates within Trusted Execution Environments to ensure computational integrity and provides censorship-resistant, verifiable trust computations.

The system works with:
- **Trust data**: Relationships between entities with trust weights
- **Seed data**: Initial trust scores for bootstrapping the algorithm
- **Score computation**: Running EigenTrust to generate final trust scores

## Installation

Build the SDK from source:

```bash
cd sdk
cargo build --release
```

The binary will be available at `target/release/openrank`.

## Commands

### Initialisation

Initialise the workspace with example datasets and .env file:

```bash
openrank init ./my-workspace
```

This command will create a folder with following structure:
- trust/
- seed/
- .env

.env file contains a placeholder for your mnemonic phrase:
```sh
MNEMONIC="add your mnemonic phrase here"
```

### Operations

#### `compute-request`
Submit a computation request using trust and seed data from local folders.

```bash
openrank compute-request <TRUST_FOLDER_PATH> <SEED_FOLDER_PATH>
```

**Arguments:**
- `TRUST_FOLDER_PATH` - Path to folder containing trust CSV files
- `SEED_FOLDER_PATH` - Path to folder containing seed CSV files

**Example:**
```bash
openrank compute-request ./trust_data ./seed_data
```

#### `compute-watch`
Monitor and watch for computation results by compute ID.

```bash
openrank compute-watch <COMPUTE_ID> [--out-dir <OUT_DIR>]
```

**Arguments:**
- `COMPUTE_ID` - The computation ID to monitor

**Options:**
- `--out-dir <OUT_DIR>` - Output directory for results (optional)

**Example:**
```bash
openrank compute-watch abc123 --out-dir ./results
```

#### `download-scores`
Download computed scores for a specific computation.

```bash
openrank download-scores <COMPUTE_ID> [--out-dir <OUT_DIR>]
```

**Arguments:**
- `COMPUTE_ID` - The computation ID to download scores for

**Options:**
- `--out-dir <OUT_DIR>` - Output directory for downloaded scores (optional)

**Example:**
```bash
openrank download-scores abc123 --out-dir ./scores
```

### Local Operations

#### `compute-local`
Run OpenRank computation locally using trust and seed CSV files.

```bash
openrank compute-local <TRUST_PATH> <SEED_PATH> [OUTPUT_PATH]
```

**Arguments:**
- `TRUST_PATH` - Path to trust CSV file
- `SEED_PATH` - Path to seed CSV file
- `OUTPUT_PATH` - Output path for computed scores (optional)

**CSV Format:**
- Trust CSV: `from_id,to_id,trust_weight`
- Seed CSV: `peer_id,score`

**Example:**
```bash
openrank compute-local trust.csv seed.csv scores.csv
```

#### `verify-local`
Verify computed scores against trust and seed data locally.

```bash
openrank verify-local <TRUST_PATH> <SEED_PATH> <SCORES_PATH>
```

**Arguments:**
- `TRUST_PATH` - Path to trust CSV file
- `SEED_PATH` - Path to seed CSV file
- `SCORES_PATH` - Path to scores CSV file to verify

**Example:**
```bash
openrank verify-local trust.csv seed.csv computed_scores.csv
```

## Data Formats

### Trust CSV Format
```csv
from_id,to_id,trust_weight
alice,bob,0.8
bob,charlie,0.6
charlie,alice,0.9
```

### Seed CSV Format
```csv
peer_id,score
alice,1.0
bob,0.5
charlie,0.3
```

### Scores CSV Format
```csv
peer_id,score
alice,0.45
bob,0.32
charlie,0.23
```

Or use AWS credential files and profiles as per standard AWS CLI configuration.

## Examples

### Complete Local Workflow

1. **Prepare your data files:**
   ```bash
   echo "alice,bob,0.8" > trust.csv
   echo "bob,charlie,0.6" >> trust.csv
   echo "alice,1.0" > seed.csv
   echo "bob,0.5" >> seed.csv
   ```

2. **Run local computation:**
   ```bash
   openrank compute-local trust.csv seed.csv scores.csv
   ```

3. **Verify the results:**
   ```bash
   openrank verify-local trust.csv seed.csv scores.csv
   ```

### Distributed Computation Workflow
1. **Submit computation request:**
   ```bash
   openrank compute-request ./trust_folder ./seed_folder
   ```

2. **Monitor computation:**
   ```bash
   openrank compute-watch <compute_id> --out-dir ./results
   ```

3. **Download results:**
   ```bash
   openrank download-scores <compute_id> --out-dir ./final_scores
   ```

## Algorithm Details

OpenRank implements the EigenTrust algorithm with the following key features:

- **Pre-trust weighting**: Uses a 0.5 weight for seed trust values
- **Convergence threshold**: Stops iteration when delta < 0.01
- **Reachability filtering**: Only includes peers reachable from seed nodes
- **Normalization**: Ensures trust distributions sum to 1.0

The algorithm iteratively computes trust scores until convergence, providing a robust measure of reputation in decentralized networks.

## Getting Help

```bash
openrank --help
openrank <command> --help
```

For more information about the OpenRank system and TEE deployment, see the main project documentation.
