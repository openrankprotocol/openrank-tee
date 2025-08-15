# -----------------------------------------------------------------------------
# This Makefile provides utilities for managing TEE AVS instances.
#
# It contains targets for:
# - Retrieving instance information and metadata
# - Fetching instance logs
#
# Default context is 'testnet'. Override with: make <target> CONTEXT=<context>
# -----------------------------------------------------------------------------

# Default context for instance operations
CONTEXT ?= testnet

# Declare phony targets
.PHONY: instance/describe instance/keys instance/logs instance/attestation help

# Get detailed information and metadata about deployed compute tee instance
instance/describe:
	./.compute-tee/scripts/describe.sh $(CONTEXT)

# Get keys of deployed compute tee instance
instance/keys:
	./.compute-tee/scripts/keys.sh $(CONTEXT)

# Get service logs of deployed compute tee instance
instance/logs:
	./.compute-tee/scripts/logs.sh $(CONTEXT)

# Get attestation quote of deployed compute tee instance
instance/attestation:
	./.compute-tee/scripts/attestation.sh $(CONTEXT)

help:
	@echo "Available targets:"
	@echo "  instance/describe: Get detailed information and metadata about deployed compute tee instance"
	@echo "  instance/keys: Get keys of deployed compute tee instance"
	@echo "  instance/logs: Get service logs of deployed compute tee instance"
	@echo "  instance/attestation: Get TDX attestation quote of deployed compute tee instance"
	@echo "  help: Show this help message"
