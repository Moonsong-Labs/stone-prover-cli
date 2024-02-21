# Stone Prover CLI

A CLI to run, prove and verify Cairo programs with a simple interface.

Features:

* Run, prove and verify any Cairo program
* Run programs directly or with the Starknet bootloader for
  compatibility with the Starknet L1 verifier
* Automatic generation of the prover configuration and parameters.

## Usage

### Run and prove a single program

```shell
stone-prover-cli prove program.json
```

### Run and prove one or more programs/PIEs with the Starknet bootloader

```shell
stone-prover-cli prove --with-bootloader program1.json program2.json pie1.zip
```

### Verify a proof

```shell
stone-prover-cli verify proof.json
```