The service exposes a single HTTP endpoint, with  Axum for the web layer and the `solana-client` non-blocking API for all blockchain interactions.


## Design
- `SlotCache` asynchronous, size-limited cache to store confirmed slot numbers. The cache maintains a limit of the last 1000 confirmed slots and with a simple eviction policy removes the oldest entries when the capacity is exceeded. Used `scc` for high-performance concurrent data access.
  
- `run_cache_poller` minimizes RPC calls by querying the `getBlocks` range from the last known confirmed slot up to the current latest slot. This ensures the cache is continuously and efficiently hydrated with confirmed data.

- `run_server` Axum server exposes the primary endpoint: `/isSlotConfirmed/{slot}`.
    - Checks the local cache first (low-latency HIT). If a miss occurs, it falls back to a direct, single-slot `getBlocks` RPC query to provide the most up-to-date status before the poller catches up.
    - Returns `200 OK` if the slot is confirmed, or `404 NOT FOUND` if it is not a confirmed block.


## Setup

### Build and Test

```bash
cargo build

cargo test
```

### Running the Service

The service requires the `RUST_LOG` environment variable to be set for visible tracing output. Use the command appropriate for your operating system's shell:

PowerShell :  `$env:RUST_LOG="info,metrics=info"; cargo run` 
Bash/Zsh (Linux/macOS) : `RUST_LOG="info,metrics=info" cargo run` 


### Testing the Endpoint

With the server running (listening on `0.0.0.0:3000`), use a new terminal to query the endpoint.

PowerShell (Windows) : `irm -Method Get http://localhost:3000/isSlotConfirmed/377335664` 
Bash/Zsh (Linux/macOS) : `curl -i http://localhost:3000/isSlotConfirmed/377335664` 
