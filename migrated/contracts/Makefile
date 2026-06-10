-include .env

.PHONY: all anvil deploy fund clean test build help

# Setup
RPC_URL := http://127.0.0.1:8545
PRIVATE_KEY := 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 # Anvil Default Account #0
BSC_RPC_URL := https://bsc-dataseed.binance.org/

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%%-20s\033[0m %%s\n", $$1, $$2}'

stop: ## Stop running Anvil node
	@lsof -i :8545 -t | xargs kill -9 2>/dev/null || echo "No Anvil process found."

anvil: ## Start Anvil forking BSC Mainnet (Blocking)
	@./start-anvil-bsc.sh

deploy: ## Deploy PaymentEscrow to local Anvil node
	@echo "Deploying PaymentEscrow to local fork..."
	@forge script script/Deploy.s.sol --rpc-url $(RPC_URL) --broadcast --private-key $(PRIVATE_KEY)

fund: ## Fund local account. Usage: make fund [address=0x...]
	@echo "Funding local account $(address)..."
	@# Default to Anvil #0 if address not set
	$(eval TARGET := $(if $(address),$(address),0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266))
	@# Calculate storage slot (Slot 1 is balance for BSC proxies)
	$(eval SLOT := $(shell cast index address $(TARGET) 1))
	@# Amount: 10,000 tokens (18 decimals) = 0x21E19E0C9BAB2400000
	@echo "Setting USDT Balance..."
	@cast rpc anvil_setStorageAt 0x55d398326f99059fF775485246999027B3197955 $(SLOT) 0x00000000000000000000000000000000000000000000021e19e0c9bab2400000 --rpc-url $(RPC_URL)
	@echo "Setting USDC Balance..."
	@cast rpc anvil_setStorageAt 0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d $(SLOT) 0x00000000000000000000000000000000000000000000021e19e0c9bab2400000 --rpc-url $(RPC_URL)
	@echo "Setting BNB Balance..."
	@cast rpc anvil_setBalance $(TARGET) 0x56BC75E2D63100000 --rpc-url $(RPC_URL)
	@echo "✅ Funded $(TARGET)"

install: ## Install dependencies
	@forge install

setup: install deploy fund ## Run install, deploy and fund (run this after starting anvil)

build: ## Build contracts
	@forge build

test: ## Run tests
	@forge test

clean: ## Clean artifacts
	@forge clean
