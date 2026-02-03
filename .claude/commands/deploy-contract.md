Deploy a smart contract to the blockchain.

Ask the user for:
1. The sender address (or use a default)
2. The assembly source code for the contract

Then call `POST /api/contracts/deploy` with the provided source code and display the contract address returned. If no source code is provided, deploy a default counter contract:

```
PUSH 0
PUSH 1
STORE
PUSH 0
LOAD
DUP
LOG
HALT
```
