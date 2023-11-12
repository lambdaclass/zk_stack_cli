# zkSync Era CLI

CLI tool for fast development on zkSync Era, built using [`zksync-web3-rs`](https://crates.io/crates/zksync-web3-rs) SDK.

## Table of Contents

- [zkSync Era CLI](#zksync-era-cli)
  - [Table of Contents](#table-of-contents)
  - [Installation](#installation)
  - [Commands](#commands)
    - [`zksync-era-cli deploy`](#zksync-era-cli-deploy)
    - [`zksync-era-cli call`](#zksync-era-cli-call)
    - [`zksync-era-cli send`](#zksync-era-cli-send)
    - [`zksync-era-cli get-contract`](#zksync-era-cli-get-contract)
    - [`zksync-era-cli get-transaction`](#zksync-era-cli-get-transaction)
    - [`zksync-era-cli balance`](#zksync-era-cli-balance)
    - [`zksync-era-cli transfer`](#zksync-era-cli-transfer)
    - [`zksync-era-cli encode`](#zksync-era-cli-encode)
    - [`zksync-era-cli selector`](#zksync-era-cli-selector)
    - [`zksync-era-cli get-bridge-contracts`](#zksync-era-cli-get-bridge-contracts)
    - [`zksync-era-cli get-bytecode-by-hash`](#zksync-era-cli-get-bytecode-by-hash)
    - [`zksync-era-cli confirmed-tokens`](#zksync-era-cli-confirmed-tokens)
    - [`zksync-era-cli l1-batch-details`](#zksync-era-cli-l1-batch-details)
    - [`zksync-era-cli l2-to-l1-log-proof`](#zksync-era-cli-l2-to-l1-log-proof)
    - [`zksync-era-cli main-contract`](#zksync-era-cli-main-contract)
    - [`zksync-era-cli deposit`](#zksync-era-cli-deposit)
    - [`zksync-era-cli withdraw`](#zksync-era-cli-withdraw)
    - [`zksync-era-cli compile`](#zksync-era-cli-compile)
      - [Status (for full compatibility) with compiler](#status-for-full-compatibility-with-compiler)

## Installation

```
git clone git@github.com:lambdaclass/zksync-era-cli.git
cd zksync-era-cli
make cli
```

## Commands

Running `zksync-era-cli` outputs the following:

```
Usage: zksync-era-cli [OPTIONS] <COMMAND>

Commands:
  deploy
  call
  get-contract
  get-transaction
  balance
  compile
  encode
  selector
  get-bridge-contracts
  get-bytecode-by-hash
  confirmed-tokens
  l1-batch-details
  l2-to-l1-log-proof
  main-contract
  transfer
  deposit
  withdraw
  send
  help                  Print this message or the help of the given subcommand(s)

Options:
      --host <HOST>        [default: localhost]
  -l, --l2-port <L2_PORT>  [default: 3050]
  -l, --l1-port <L1_PORT>  [default: 8545]
  -h, --help               Print help
  -V, --version            Print version
```

### `zksync-era-cli deploy`

Deploys a contract, this can be done in two ways:

#### Deploying from contract path

Deploys a contract, the project root path (`PROJECT_PATH`), the `CONTRACT_PATH` and the `CONTRACT_NAME` must be specified. The `PRIVATE_KEY` is needed to sign the deploy transaction.

```
zksync-era-cli deploy --project-root <PROJECT_PATH> --contract <CONTRACT_PATH> --contract-name <CONTRACT_NAME> --private-key <PRIVATE_KEY> 
```

#### Deploying from a compiled contract (artifact)

Deploys a contract that was previously compiled and saved as an artifact. The `PRIVATE_KEY` is needed to sign the deploy transaction.

```
zksync-era-cli deploy --contract_artifact <CONTRACT_NAME> --private-key <PRIVATE_KEY> 
```

### `zksync-era-cli call`

Calls `FUNCTION_SIGNATURE` of `CONTRACT_ADDRESS` with args `FUNCTION_ARGS` or the corresponding calldata instead using the `--data` flag. Use this command to call a `public view` contract function, if the contract function performs a state change the send command must be used.

```
zksync-era-cli call --contract <CONTRACT_ADDRESS> --function <FUNCTION_SIGNATURE> --args <FUNCTION_ARGS>
```

### `zksync-era-cli send`

Sends a transaction to a function which modifies the state with signature `FUNCTION_SIGNATURE` of `CONTRACT_ADDRESS` with args `FUNCTION_ARGS` or the corresponding calldata instead using the `--data` flag. The transaction will be signed with the sender `PRIVATE_KEY`.

```
zksync-era-cli send --contract <CONTRACT_ADDRESS> --function <FUNCTION_SIGNATURE> --private-key <PRIVATE_KEY> --chain-id <CHAIN_ID>
```

### `zksync-era-cli get-contract`

Gets `CONTRACT_ADDRESS`'s bytecode.

```
zksync-era-cli get-contract --contract <CONTRACT_ADDRESS>
```

### `zksync-era-cli get-transaction`

Get the transaction corresponding to `TRANSACTION_HASH`.

```
zksync-era-cli get-transaction --transaction <TRANSACTION_HASH>
```

### `zksync-era-cli balance`

Gets the balance of the `ACCOUNT_ADDRESS`.

```
zksync-era-cli balance --account <ACCOUNT_ADDRESS>
```

### `zksync-era-cli transfer`

Transfer `AMOUNT` from `SENDER_PRIVATE_KEY` to `RECEIVER_ADDRESS` signing the transaction with senders private key.

```
zksync-era-cli transfer --amount <AMOUNT_TO_TRANSFER> --from <SENDER_PRIVATE_KEY> --to <RECEIVER_ADDRESS>
```

### `zksync-era-cli encode`

Encodes a function with signature `FUNCTION_SIGNATURE` and arguments `ARGUMENTS` which types are `ARG_TYPES`.

```
zksync-era-cli encode --function <FUNCTION_SIGNATURE> --arguments <ARGUMENTS> --types <ARG_TYPES>
```

### `zksync-era-cli selector`

Encodes the function signature `FUNCTION_SIGNATURE` into the function selector.

```
zksync-era-cli selector --function-signature <FUNCTION_SIGNATURE>
```

### `zksync-era-cli get-bridge-contracts`

Returns the addresses of the bridge contracts in L1 and L2 nodes.

```
zksync-era-cli get-bridge-contracts
```

### `zksync-era-cli get-bytecode-by-hash`

Returns the contract bytecode from its hash `CONTRACT_BYTECODE_HASH`

```
zksync-era-cli get-bytecode-by-hash --hash <CONTRACT_BYTECODE_HASH>
```

### `zksync-era-cli confirmed-tokens`

Returns address, symbol, name, and decimal information of all tokens within a range of ids starting in `FROM` to `FROM` + `LIMIT`

Confirmd in this context means any token bridged to zkSync via the official bridge.

```
zksync-era-cli confirmed-tokens --from <FROM> --limit <LIMIT>
```

### `zksync-era-cli l1-batch-details`

Returns data pertaining to the batch with number `L1_BATCH_NUMBER`.

```
zksync-era-cli confirmed-tokens --from <FROM> --limit <LIMIT>
```

### `zksync-era-cli l2-to-l1-log-proof`

This command takes two possible flags, `--log-proof` or `--msg-proof`. One of them must be present.

If the command is running with `--log-proof` command it
gets the proof for the corresponding L2 to L1 log a transaction with hash `TRANSACTION_HASH`, and the index `LOG_INDEX` of the L2 to L1 log produced within the transaction.

If the command is running with `--msg-proof` command it
gets the proof for the message sent via the L1Messenger system contract with sender address `MESSAGE_SENDER`, the message `MESSAGE` and block number `MESSAGE_BLOCK`.

```
zksync-era-cli l2-to-l1-log-proof --transaction <TRANSACTION_HASH> --block <MESSAGE_BLOCK> --sender <MESSAGE_SENDER> --msg <MESSAGE> --log-index <LOG_INDEX>
```
### `zksync-era-cli main-contract`

Returns the address of the zkSync Era contract.

```
zksync-era-cli main-contract
```

### `zksync-era-cli deposit`

Performs a deposit for an amount `AMOUNT_TO_DEPOSIT_IN_ETHER` from the L1 account with private key `SENDER_PRIVATE_KEY` and L1 chain id `CHAIN_ID` to the account with the same address in L2. In case the address from L1 and L2 were different the L2 address can be specified with the `--to` argument.

```
zksync-era-cli deposit --amount <AMOUNT_TO_DEPOSIT_IN_ETHER> --from <SENDER_PRIVATE_KEY> --chain-id <CHAIN_ID>
```

### `zksync-era-cli withdraw`

Performs a withdraw for an amount `AMOUNT_TO_WITHDRAW_IN_ETHER` from the L2 account with private key `SENDER_PRIVATE_KEY` and L2 chain id `CHAIN_ID` to the account with the same address in L1. In case the address from L2 and L1 were different the L1 address can be specified with the `--to` argument.

```
zksync-era-cli withdraw --amount <AMOUNT_TO_WITHDRAW_IN_ETHER> --from <SENDER_PRIVATE_KEY> --chain-id <CHAIN_ID>
```

### `zksync-era-cli compile`

Compiles a contract, using the binary located in `COMPILER_PATH` (zksolc or solc), contained in the project with path `PROJECT_ROOT_PATH` with the corresponding path `CONTRACT_PATH` and contract name being `CONTRACT_NAME`.

```
zksync-era-cli compile --compiler <COMPILER_PATH> --project-root <PROJECT_ROOT_PATH> --contract-path <CONTRACT_PATH> --contract-name <CONTRACT_NAME>
```

#### Status (for full compatibility) with compiler

| Flags                      | Description                                                                                                                                                                                                                                                 | Supported | State |
| -------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ----- |
| `--disable-solc-optimizer` | Disable the `solc` optimizer. Use it if your project uses the `MSIZE` instruction, or in other cases. Beware that it will prevent libraries from being inlined                                                                                              | ❌        | ❌    |
| `--force-evmla`            | Forcibly switch to the EVM legacy assembly pipeline. It is useful for older revisions of `solc` 0.8, where Yul was considered highly experimental and contained more bugs than today                                                                        | ❌        | ❌    |
| `-h, --help`               | Prints help information                                                                                                                                                                                                                                     | ✅        | ✅    |
| `--system-mode`            | Enable the system contract compilation mode. In this mode zkEVM extensions are enabled. For example, calls to addresses `0xFFFF` and below are substituted by special zkEVM instructions. In the Yul mode, the `verbatim_*` instruction family is available | ❌        | ❌    |
| `--llvm-debug-logging`     | Set the debug-logging option in LLVM. Only for testing and debugging                                                                                                                                                                                        | ❌        | ❌    |
| `--llvm-ir`                | Switch to the LLVM IR mode. Only one input LLVM IR file is allowed. Cannot be used with the combined and standard JSON modes                                                                                                                                | ❌        | ❌    |
| `--llvm-verify-each`       | Set the verify-each option in LLVM. Only for testing and debugging                                                                                                                                                                                          | ❌        | ❌    |
| `--asm`                    | Output zkEVM assembly of the contracts                                                                                                                                                                                                                      | ❌        | ❌    |
| `--bin`                    | Output zkEVM bytecode of the contracts                                                                                                                                                                                                                      | ❌        | ❌    |
| `--overwrite`              | Overwrite existing files (used together with -o)                                                                                                                                                                                                            | ❌        | ❌    |
| `--standard-json`          | Switch to standard JSON input/output mode. Read from stdin, write the result to stdout. This is the default used by the hardhat plugin                                                                                                                      | ❌        | 🏗     |
| `--version`                | Print the version and exit                                                                                                                                                                                                                                  | ❌        | ❌    |
| `--yul`                    | Switch to the Yul mode. Only one input Yul file is allowed. Cannot be used with the combined and standard JSON modes                                                                                                                                        | ❌        | ❌    |

| Options                                       | Description                                                                                                                                                                                                                      | Supported | State |
| --------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- | ----- |
| `--allow-paths <allow-paths>`                 | Allow a given path for imports. A list of paths can be supplied by separating them with a comma. Passed to `solc` without changes                                                                                                | ❌        | ❌    |
| `--base-path <base-path>`                     | Set the given path as the root of the source tree instead of the root of the filesystem. Passed to `solc` without changes                                                                                                        | ❌        | ❌    |
| `--combined-json <combined-json>`             | Output a single JSON document containing the specified information. Available arguments: `abi`, `hashes`, `metadata`, `devdoc`, `userdoc`, `storage-layout`, `ast`, `asm`, `bin`, `bin-runtime`                                  | ✅        | ✅    |
| `--debug-output-dir <debug-output-directory>` | Dump all IRs to files in the specified directory. Only for testing and debugging                                                                                                                                                 | ❌        | ❌    |
| `--include-path <include-paths>...`           | Make an additional source directory available to the default import callback. Can be used multiple times. Can only be used if the base path has a non-empty value. Passed to `solc` without changes                              | ❌        | ❌    |
| `-l, --libraries <libraries>...`              | Specify addresses of deployable libraries. Syntax: `<libraryName>=<address> [, or whitespace] ...`. Addresses are interpreted as hexadecimal strings prefixed with `0x`                                                          | ❌        |   ❌    |
| `--metadata-hash <metadata-hash>`             | Set the metadata hash mode. The only supported value is `none` that disables appending the metadata hash. Is enabled by default                                                                                                  | ❌        |   ❌    |
| `-O, --optimization <optimization>`           | Set the optimization parameter -O\[0 \| 1 \| 2 \| 3 \| s \| z\]. Use `3` for best performance and `z` for minimal size                                                                                                           | ❌        |      ❌ |
| `-o, --output-dir <output-directory>`         | Create one file per component and contract/file at the specified directory, if given                                                                                                                                             | ❌        |      ❌ |
| `--solc <solc>`                               | Specify the path to the `solc` executable. By default, the one in `${PATH}` is used. Yul mode: `solc` is used for source code validation, as `zksolc` itself assumes that the input Yul is valid. LLVM IR mode: `solc` is unused | ✅        |  🏗     |
