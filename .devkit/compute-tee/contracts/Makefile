# Configuration variables
VERIFY ?= true

# Build the project
.PHONY: build
build:
	forge clean
	forge build

# Test the project
.PHONY: test
test:
	forge test -vvv

# Deploy main contracts using environment variables as input
.PHONY: deploy-from-env
deploy-from-env:
	forge script script/Deploy.s.sol \
		--sig "run((string,string,address,address,address,address,address,address,address,address,uint32,string))" \
		'("$(ENVIRONMENT)","$(VERSION)",$(DELEGATION_MANAGER),$(ALLOCATION_MANAGER),$(PERMISSION_CONTROLLER),$(KEY_REGISTRAR),$(RELEASE_MANAGER),$(PROXY_ADMIN),$(INITIAL_OWNER),$(DELEGATION_APPROVER),$(ALLOCATION_DELAY),"$(METADATA_URI)")' \
		--rpc-url $(RPC_URL) \
		--broadcast \
		--private-key $(PRIVATE_KEY) \
		$(if $(filter true,$(VERIFY)),--verify --verifier etherscan --etherscan-api-key $(ETHERSCAN_API_KEY)) \
		--slow \
		-vvvv

# Deploy main contracts from using config/deploy.json as input
.PHONY: deploy-from-config
deploy-from-config:
	forge script script/Deploy.s.sol \
		--sig "run()" \
		--rpc-url $(RPC_URL) \
		--broadcast \
		--private-key $(PRIVATE_KEY) \
		$(if $(filter true,$(VERIFY)),--verify --verifier etherscan --etherscan-api-key $(ETHERSCAN_API_KEY)) \
		--slow \
		-vvvv

# Create AVS instance
.PHONY: create-avs
create-avs:
	forge script script/CreateAVS.s.sol \
		--rpc-url $(RPC_URL) \
		--broadcast \
		--sig "run(string, string, address, address)" $(ENVIRONMENT) $(METADATA_URI) $(PERMISSION_CONTROLLER) $(TEE_AVS_FACTORY) \
		--private-key $(PRIVATE_KEY_DEPLOYER) \
		--slow \
		-vvvv

# Helper message
.PHONY: help
help:
	@echo "Available commands:"
	@echo "  make build - Build the project"
	@echo "  make test - Test the project"
	@echo "  make deploy-from-env - Deploy main contracts using environment variables as input"
	@echo "  make deploy-from-config - Deploy main contracts from using config/deploy.json as input"
	@echo "  make create-avs - Create AVS instance using CreateAVS.s.sol"
	@echo ""
	@echo "Configuration options:"
	@echo "  VERIFY=true|false - Enable/disable Etherscan verification (default: true)"
	@echo "    Example: make deploy-from-env VERIFY=false"
	@echo ""
	@echo "Note: Make sure to set RPC_URL and PRIVATE_KEY in your environment or .env file" 
