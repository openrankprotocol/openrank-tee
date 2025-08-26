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

The binary will be available at `target/release/openrank-sdk`.

## Commands

### Meta Operations

#### `meta-compute-request`
Submit a computation request using trust and seed data from local folders.

```bash
openrank-sdk meta-compute-request <TRUST_FOLDER_PATH> <SEED_FOLDER_PATH>
```

**Arguments:**
- `TRUST_FOLDER_PATH` - Path to folder containing trust CSV files
- `SEED_FOLDER_PATH` - Path to folder containing seed CSV files

**Example:**
```bash
openrank-sdk meta-compute-request ./trust_data ./seed_data
```

#### `meta-compute-watch`
Monitor and watch for computation results by compute ID.

```bash
openrank-sdk meta-compute-watch <COMPUTE_ID> [--out-dir <OUT_DIR>]
```

**Arguments:**
- `COMPUTE_ID` - The computation ID to monitor

**Options:**
- `--out-dir <OUT_DIR>` - Output directory for results (optional)

**Example:**
```bash
openrank-sdk meta-compute-watch abc123 --out-dir ./results
```

#### `meta-download-scores`
Download computed scores for a specific computation.

```bash
openrank-sdk meta-download-scores <COMPUTE_ID> [--out-dir <OUT_DIR>]
```

**Arguments:**
- `COMPUTE_ID` - The computation ID to download scores for

**Options:**
- `--out-dir <OUT_DIR>` - Output directory for downloaded scores (optional)

**Example:**
```bash
openrank-sdk meta-download-scores abc123 --out-dir ./scores
```

### Local Operations

#### `compute-local`
Run OpenRank computation locally using trust and seed CSV files.

```bash
openrank-sdk compute-local <TRUST_PATH> <SEED_PATH> [OUTPUT_PATH]
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
openrank-sdk compute-local trust.csv seed.csv scores.csv
```

#### `verify-local`
Verify computed scores against trust and seed data locally.

```bash
openrank-sdk verify-local <TRUST_PATH> <SEED_PATH> <SCORES_PATH>
```

**Arguments:**
- `TRUST_PATH` - Path to trust CSV file
- `SEED_PATH` - Path to seed CSV file
- `SCORES_PATH` - Path to scores CSV file to verify

**Example:**
```bash
openrank-sdk verify-local trust.csv seed.csv computed_scores.csv
```

### Data Management

#### `upload-trust`
Upload trust data to the distributed storage system.

```bash
openrank-sdk upload-trust <PATH> <CERTS_PATH>
```

**Arguments:**
- `PATH` - Path to trust CSV file
- `CERTS_PATH` - Path to certificates for authentication

**Example:**
```bash
openrank-sdk upload-trust trust.csv ./certs
```

#### `download-trust`
Download trust data from the distributed storage system.

```bash
openrank-sdk download-trust <PATH> <CERTS_PATH>
```

**Arguments:**
- `PATH` - Local path to save downloaded trust data
- `CERTS_PATH` - Path to certificates for authentication

**Example:**
```bash
openrank-sdk download-trust ./downloaded_trust.csv ./certs
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

## Environment Setup

The SDK requires AWS credentials for S3 operations. Set up your environment:

```bash
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_DEFAULT_REGION=your_region
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
   openrank-sdk compute-local trust.csv seed.csv scores.csv
   ```

3. **Verify the results:**
   ```bash
   openrank-sdk verify-local trust.csv seed.csv scores.csv
   ```

### Distributed Computation Workflow
1. **Submit computation request:**
   ```bash
   openrank-sdk meta-compute-request ./trust_folder ./seed_folder
   ```

2. **Monitor computation:**
   ```bash
   openrank-sdk meta-compute-watch <compute_id> --out-dir ./results
   ```

3. **Download results:**
   ```bash
   openrank-sdk meta-download-scores <compute_id> --out-dir ./final_scores
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
openrank-sdk --help
openrank-sdk <command> --help
```

For more information about the OpenRank system and TEE deployment, see the main project documentation.
