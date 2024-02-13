.PHONY: deps all-tests clean distclean unit-tests integration-tests

PROVER_BIN=dependencies/cpu_air_prover
VERIFIER_BIN=dependencies/cpu_air_verifier

$(PROVER_BIN):
	wget -O dependencies/cpu_air_prover https://github.com/Moonsong-Labs/stone-prover-sdk/releases/download/v0.1.0-rc1/cpu_air_prover

$(VERIFIER_BIN):
	wget -O dependencies/cpu_air_verifier https://github.com/Moonsong-Labs/stone-prover-sdk/releases/download/v0.1.0-rc1/cpu_air_verifier

all-tests: deps
	cargo test

itests: deps
	cargo test --release --test '*'

deps: $(PROVER_BIN) $(VERIFIER_BIN)
clean:
	cargo clean

distclean: clean
	rm -rf dependencies/cpu_air_prover
	rm -rf dependencies/cpu_air_verifier
