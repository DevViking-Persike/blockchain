Run a full API test suite against the running node.

Test all 12 endpoints in order:
1. `GET /api/node/info`
2. `GET /api/chain/valid`
3. `GET /api/peers`
4. `POST /api/wallet/new` - Create a wallet
5. `POST /api/blocks/mine` - Mine a block
6. `GET /api/balance/miner-node` - Check miner balance
7. `GET /api/blocks/0` - Get genesis block
8. `GET /api/blocks/1` - Get mined block
9. `GET /api/transactions/pending` - Pending transactions
10. `POST /api/contracts/deploy` with source: `PUSH 10\nPUSH 20\nADD\nDUP\nLOG\nHALT`
11. `POST /api/contracts/call` on the deployed contract
12. `GET /api/chain` - Full chain

Display each result clearly with the endpoint name.
