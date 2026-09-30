# Makefile for Standalone Quantum Hilbert Hotel Ledger Microservice

CARGO := cargo
DAEMON_BIN := qcos-hilbert-ledger
SOCKET_PATH := /tmp/qcos_hilbert_ledger.sock
CONFIG_PATH := config/ledger_daemon.toml

.PHONY: all build run test clean docker-build docker-run client-test

all: build

# Compile release binary for the ledger daemon and runtime binaries
build:
	@echo "=== Compiling Standalone Hilbert Ledger Daemon ==="
	$(CARGO) build --release

# Run the daemon locally over UNIX domain socket
run: build
	@echo "=== Starting Standalone Daemon ==="
	@rm -f $(SOCKET_PATH)
	./target/release/$(DAEMON_BIN)

# Run full integration test suite (AgentQ & GUS EAP verification)
test:
	@echo "=== Running Hilbert Ledger Integration Tests ==="
	$(CARGO) test -- --nocapture

# Run the dual AgentQ + GUS live runtime integration test
runtime-test: build
	@echo "=== Executing AgentQ & GUS Bridge Runtime ==="
	$(CARGO) run --release --bin agent_gus_runtime

# Test Python Client Bridge (IPC test with AgentQ/GUS client IDs)
python-test:
	@echo "=== Executing Python IPC Bridge Test ==="
	python3 clients/hilbert_client.py

# Build isolated Docker container
docker-build:
	@echo "=== Building Standalone Hilbert Ledger Docker Image ==="
	docker build -f Dockerfile.standalone -t qcos/hilbert-ledger:standalone .

# Run Docker container with host socket volume mount
docker-run:
	@echo "=== Running Standalone Hilbert Ledger Container ==="
	docker run --rm -it \
		-v /tmp:/tmp \
		--name qcos-hilbert-ledger-daemon \
		qcos/hilbert-ledger:standalone

# Clean build artifacts and stale socket files
clean:
	@echo "=== Cleaning Build Artifacts & IPC Sockets ==="
	$(CARGO) clean
	rm -f $(SOCKET_PATH)