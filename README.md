# ZKsync CLI

`zks` is a versatile command-line interface (CLI) tool that serves as both an interface for interacting with a ZKsync chain and a powerful tool for managing it. By wrapping the ZKsync JSON-RPC API, `zks` offers a more intuitive and efficient experience, whether you're working with smart contracts or managing assets.

With `zks`, you can seamlessly perform a variety of tasks on the ZKsync chain, including:

- **Depositing tokens** from Layer 1 (L1) to ZKsync.
- **Withdrawing tokens** from ZKsync back to L1.
- **Transferring tokens** between Layer 2 (L2) accounts.
- **Compiling, deploying, and interacting with contracts** directly on ZKsync.

Whether you're a developer focused on deploying and interacting with contracts or a user managing your tokens, `zks` empowers you to handle both the interaction and management aspects of the ZKsync ecosystem with ease.

## Table of Contents

- [Table of Contents](#table-of-contents)
- [Installation](#installation)
- [Usage](#usage)
  - [Config](#config)
  - [Wallet](#wallet)
  - [Chain](#chain)
  - [Contract](#contract)
  - [Contracts](#contracts)
  - [Prover](#prover)
  - [Test](#test)
- [Features](#features)
  - [ZKsync JSON-RPC API](#zksync-json-rpc-api)
  - [ZKsync SDK](#zksync-sdk)

## Installation

```
git clone git@github.com:lambdaclass/zksync_era_cli.git
cd zksync_era_cli
make cli
```

## Usage

Running `zks` outputs the following:

```
Usage: zks <COMMAND>

Commands:
  wallet     Wallet interaction commands. The configured wallet could operate both with the L1 and L2 networks.
  chain      Chain interaction commands. These make use of the JSON-RPC API.
  prover     Prover commands. TODO.
  contract   Contract interaction commands.
  config     CLI config commands.
  contracts  L1 Contracts interaction commands. For the chain owner. [aliases: l1]
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Config

The configuration is strictly necessary to interact with the CLI without issues. **All the commands but the `config` require a configuration to be set**. The configuration is stored in `.toml` files in the user's config directory (`~/.config/zks-cli/<you_config_name>.toml`) and looks like this:

```toml
[network]
l1_rpc_url=""
l1_explorer_url=""
l2_rpc_url=""
l2_explorer_url=""

[wallet]
address=""
private_key=""
```

This command is in charge of managing the configuration the CLI will use for all the other commands. Running `zks config` shows you the available commands:

```
CLI config commands.

Usage: zks config <COMMAND>

Commands:
  edit     Edit an existing config.
  create   Create a new config.
  set      Set the config to use.
  display  Display a config.
  list     List all configs.
  delete   Delete a config.
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

You can create multiple configs as "profiles" to be able to use the tool with for example various servers, or wallets without having to use different commands with different flags or have to manually edit one config file each time.

#### `zks config create`

A command used to create new configurations interactively.

```
Create a new config.

Usage: zks config create --name <CONFIG_NAME>

Options:
      --name <CONFIG_NAME>
  -h, --help                Print help
```

#### `zks config edit`

Used for editing existing configurations. You can do this with flags to edit specific values or you can edit it interactively.

```
Edit an existing config.

Usage: zks config edit [OPTIONS]

Options:
      --name <CONFIG_NAME>
      --l1-rpc-url <L1_RPC_URL>
      --l2-rpc-url <L2_RPC_URL>
      --l2-explorer-url <L2_EXPLORER_URL>
      --l1-explorer-url <L1_EXPLORER_URL>
      --private-key <PRIVATE_KEY>
      --address <ADDRESS>
  -e, --interactively
  -h, --help                               Print help
```

#### `zks config set`

Once you've created one or multiple configs it is time to set which one you're going to use. For this, you can use the `zks config set` command which lets you choose interactively, which config among the created ones you want to set; or you can set it by name.

```
Set the config to use.

Usage: zks config set [OPTIONS] --name <CONFIG_NAME>

Options:
      --name <CONFIG_NAME>
  -s, --interactively
  -h, --help                Print help
```

#### `zks config display`

If you want to see what configuration is set in one specific config profile, use `zks config display` to display a config file by name or choose it interactively among the existing config files.

```
Display a config.

Usage: zks config display [OPTIONS] --name <CONFIG_NAME>

Options:
      --name <CONFIG_NAME>
  -s, --interactively
  -h, --help                Print help
```

#### `zks config list`

Lists all the existing configurations.

```
List all configs.

Usage: zks config list

Options:
  -h, --help  Print help
```

#### `zks config delete`

Deletes an existing config. You can either delete by name or choose interactively among the existing configs.

```
Delete a config.

Usage: zks config delete [OPTIONS]

Options:
      --name <CONFIG_NAME>
  -d, --interactively
  -h, --help                Print help
```

### Wallet

```
Wallet interaction commands. The configured wallet could operate both with the L1 and L2 networks.

Usage: zks wallet <COMMAND>

Commands:
  balance            Get the balance of the wallet.
  deposit            Deposit funds into the wallet.
  finalize-withdraw  Finalize a pending withdrawal.
  transfer           Transfer funds to another wallet.
  withdraw           Withdraw funds from the wallet.
  address            Get the wallet address.
  private-key        Get the wallet private key.
  help               Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Chain

```
Chain interaction commands. These make use of the JSON-RPC API.

Usage: zks chain <COMMAND>

Commands:
  get-code               Get the deployed bytecode of a contract
  get-transaction        Get a transaction by hash
  bridge-contracts       Retrieves the addresses of canonical bridge contracts for ZKsync Era.
  get-bytecode-by-hash   Retrieves the bytecode of a transaction by its hash.
  confirmed-tokens       Lists confirmed tokens. Confirmed in the method name means any token bridged to ZKsync Era via the official bridge.
  l1-batch-details       Retrieves details for a given L1 batch.
  l2-to-l1-log-proof
  main-contract          Retrieves the main contract address.
  bridgehub-contract     Retrieves the bridge hub contract address.
  testnet-paymaster      Retrieves the testnet paymaster address, specifically for interactions within the ZKsync Sepolia Testnet environment. Note: This method is only applicable for ZKsync Sepolia Testnet.
  l1-chain-id            Retrieves the L1 chain ID.
  l1-base-token-address  Retrieves the L1 base token address.
  all-account-balances   Gets all account balances for a given address.
  l1-batch-number        Retrieves the current L1 batch number.
  block-details          Retrieves details for a given block.
  transaction-details    Retrieves details for a given transaction.
  l1-gas-price           Retrieves the current L1 gas price.
  fee-params             Retrieves the current fee parameters.
  protocol-version       Gets the protocol version.
  balance                Get the balance of an account.
  finalize-deposit-tx    Gets the finalize deposit transaction hash.
  help                   Print this message or the help of the given subcommand(s)

Options:
  -h, --helpPrint help
```

### Contract

```
Contract interaction commands.

Usage: zks contract <COMMAND>

Commands:
  call    Call view functions on a contract.
  deploy  Deploy a contract.
  send    Call non-view functions on a contract.
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Contracts

```
L1 Contracts interaction commands. For the chain owner.

Usage: zks contracts <COMMAND>

Commands:
  bridgehub                 Bridgehub contract interaction commands. [aliases: bh]
  governance                Governance contract interaction commands. [aliases: g]
  hyperchain                Hyperchain contract interaction commands. [aliases: h]
  l1-shared-bridge          L1SharedBridge contract interaction commands. [aliases: l1sb]
  state-transition-manager  StateTransitionManager contract interaction commands. [aliases: stm]
  help                      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

#### Bridgehub

```
Bridgehub contract interaction commands.

Usage: zks contracts bridgehub <COMMAND>

Commands:
  state-transition-manager  Get the StateTransitionManager contract address of a chain. [aliases: stm]
  base-token                Get the base token contract of a chain. [aliases: bt]
  admin                     Get the bridge contract admin address.
  set-pending-admin         Set a new admin of the Bridgehub. Only the Bridgehub owner or the current admin can do this. [aliases: spa]
  accept-admin              Accept the admin of the Bridgehub. Only the pending admin can do this. [aliases: aa]
  get-hyperchain            Get the Hyperchain contract address of a chain. [aliases: h, hyperchain]
  help                      Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

#### Governance

```
Governance contract interaction commands.

Usage: zks contracts governance <COMMAND>

Commands:
  is-operation             Returns whether an id corresponds to a registered operation. This includes Waiting, Ready, and Done operations.
  is-operation-pending     Returns whether an operation is pending or not. Note that a "pending" operation may also be "ready".
  is-operation-ready       Returns whether an operation is ready for execution. Note that a "ready" operation is also "pending".
  is-operation-done        Returns whether an operation is done or not.
  operation-state          Returns the state of an operation.
  propose-upgrade          Propose an upgrade, this could be fully transparent providing upgrade data on-chain, or a "shadow" upgrade not publishing data on-chain. Only the current owner can propose a shadow upgrade.
  cancel                   Cancel a scheduled operation.
  execute                  Execute a scheduled operation.
  hash-operation           Get the hash of an operation.
  update-min-delay         Changes the minimum timelock duration for future operations. [aliases: ud]
  update-security-council  Updates the address of the security council. [aliases: usc]
  help                     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

#### Hyperchain

TODO

#### L1 Shared Bridge

TODO

#### State Transition Manager

TODO

### Prover

TODO

### Test

```
Test commands, LoadTests, Benchmarks, etc.

Usage: zks test <COMMAND>

Commands:
  load-test      LoadTest the zkStack Chain.
  erc20-l1-mint  Mint ERC20 token on L1.
  help           Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

#### ` zks test load-test`

It will make use of the wallet set specified in the config file, if we want to run the loadtest in a loop with 10 random wallets and with `4` BaseToken Tokens:

`-r` means reruns, 0 stands for a infinite runs.

```sh
zks test loadtest --wallets 10 --amount 4 -r 0
```

## Features

### ZKsync JSON-RPC API 

|                   Command                   |                  Endpoint                  | Status |
| :-----------------------------------------: | :----------------------------------------: | :----: |
|               `estimate-fee`                |             `zks_estimateFee`              |   🏗️    |
|           `estimate-gas-l1-to-l2`           |          `zks_estimateGasL1ToL2`           |   🏗️    |
|            `bridgehub-contract`             |         `zks_getBridgehubContract`         |   ✔️    |
|               `main-contract`               |           `zks_getMainContract`            |   ✔️    |
|             `testnet-paymaster`             |         `zks_getTestnetPaymaster`          |   ✔️    |
|             `bridge-contracts`              |          `zks_getBridgeContracts`          |   ✔️    |
|                `l1-chain-id`                |             `zks_getL1ChainId`             |   ✔️    |
|           `l1-base-token-address`           |        `zks_getL1BaseTokenAddress`         |   ✔️    |
|             `confirmed-tokens`              |          `zks_getConfirmedTokens`          |   ✔️    |
|           `all-account-balances`            |        `zks_getAllAccountBalances`         |   ✔️    |
|                     ``                      |          `zks_getL2ToL1MsgProof`           |   🏗️    |
|                     ``                      |          `zks_getL2ToL1LogProof`           |   🏗️    |
|              `l1-batch-number`              |           `zks_getL1BatchNumber`           |   ✔️    |
|               `block-details`               |           `zks_getBlockDetails`            |   ✔️    |
|            `transaction-details`            |        `zks_getTransactionDetails`         |   ✔️    |
|          `raw-blocks-transactions`          |       `zks_getRawBlocksTransactions`       |   ❌    |
|             `l1-batch-details`              |          `zks_getL1BatchDetails`           |   ✔️    |
|             `bytecode-by-hash`              |          `zks_getBytecodeByHash`           |   ✔️    |
|              `l1-block-range`               |           `zks_getL1BlockRange`            |   🏗️    |
|               `l1-gas-price`                |            `zks_getL1GasPrice`             |   ✔️    |
|                `fee-params`                 |             `zks_getFeeParams`             |   ✔️    |
|             `protocol-version`              |          `zks_getProtocolVersion`          |   ✔️    |
|                   `proof`                   |               `zks_getProof`               |   🏗️    |
| `send-raw-transaction-with-detailed-output` | `zks_sendRawTransactionWithDetailedOutput` |   ❌    |

### ZKsync SDK

|        Command        |            Feature            | Status |
| :-------------------: | :---------------------------: | :----: |
|       `deploy`        |       Deploy a contract       |   🏗️    |
|        `call`         |        Call a contract        |   🏗️    |
|        `send`         |      Send a transaction       |   🏗️    |
|       `balance`       | Get the balance of an account |   ✔️    |
|   `transfer` ERC20    |        Transfer funds         |   🏗️    |
| `transfer` Base Token |        Transfer funds         |   ✔️    |
|       `compile`       |      Compile a contract       |   🏗️    |
| `deposit` Base Token  |         Deposit funds         |   ✔️    |
|    `deposit` ERC20    |         Deposit funds         |   🏗️    |
|      `withdraw`       |        Withdraw funds         |   🏗️    |
