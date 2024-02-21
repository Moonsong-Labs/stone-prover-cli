# Stone Prover CLI

A CLI to run, prove and verify Cairo programs with a simple interface.

Features:

* Run, prove and verify any Cairo program
* Run programs directly or with the Starknet bootloader for
  compatibility with the Starknet L1 verifier
* Automatic generation of the prover configuration and parameters.

## Install

Install dependencies. `libdw1` is required to run Stone and `wget` to download the installation script.

```shell
# For Debian/Ubuntu
sudo apt install libdw1 wget
```

```shell
wget -O - https://raw.githubusercontent.com/Moonsong-Labs/stone-prover-cli/main/scripts/install-stone-cli.sh | bash
```

For now, only Linux platforms are supported.

## Usage

### Run and prove a single program

After compiling a Cairo0 program to `program.json`, run:

```shell
stone-prover-cli prove program.json
```

### Run and prove one or more programs/PIEs with the Starknet bootloader

If you want to prove one or more programs and PIEs by running them with the Starknet bootloader,
you can use the `--with-bootloader` option.

```shell
stone-prover-cli prove --with-bootloader program1.json program2.json pie1.zip
```

### Verify a proof

If you want to verify the generated proof file, run:

```shell
stone-prover-cli verify proof.json
```