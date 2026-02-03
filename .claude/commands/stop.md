Stop the running blockchain node.

Kill any process listening on the API port (default 8080) using `lsof -ti:8080 | xargs kill -9`. Also stop any background bash tasks related to the node. Confirm the node was stopped.
