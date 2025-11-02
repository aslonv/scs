## Overview

The service exposes a single HTTP endpoint, with  **Axum** for the web layer and the **`solana-client`** non-blocking API for all blockchain interactions.

---

## Architecture and Design

1. **Cache Layer (`SlotCache`):** An asynchronous, size-limited cache to store confirmed slot numbers. The cache maintains a limit of the last **1000 confirmed slots** and with a simple eviction policy removes the oldest entries when the capacity is exceeded. Used **`scc`** for high-performance concurrent data access.
2. **RPC Poller (`run_cache_poller`):** The minimizes RPC calls by querying the `getBlocks` range from the last known confirmed slot up to the current latest slot. This ensures the cache is continuously and efficiently hydrated with confirmed data.
3. **HTTP Service (`run_server`):** The Axum server exposes the primary endpoint: `/isSlotConfirmed/{slot}`.
    * **Control Flow:** it checks the local cache first (low-latency HIT). If a miss occurs, it falls back to a direct, single-slot `getBlocks` RPC query to provide the most up-to-date status before the poller catches up.
    * **Response:** Returns `200 OK` if the slot is confirmed, or `404 NOT FOUND` if it is not a confirmed block.

---

## Setup and Execution

### Build and Test

```bash
cargo build

cargo test
```

### Running the Service

The service requires the `RUST_LOG` environment variable to be set for visible tracing output. Use the command appropriate for your operating system's shell:

| Shell | Command |
| :--- | :--- |
| **PowerShell (Windows)** | `$env:RUST_LOG="info,metrics=info"; cargo run` |
| **Bash/Zsh (Linux/macOS)** | `RUST_LOG="info,metrics=info" cargo run` |


### Testing the Endpoint

With the server running (listening on `0.0.0.0:3000`), use a new terminal to query the endpoint.

| Shell | Command (Example for Confirmed Slot) |
| :--- | :--- |
| **PowerShell (Windows)** | `irm -Method Get http://localhost:3000/isSlotConfirmed/377335664` |
| **Bash/Zsh (Linux/macOS)** | `curl -i http://localhost:3000/isSlotConfirmed/377335664` |

*Note: On Windows, the PowerShell `curl` alias often prompts for parameters. For reliable API testing, use the native cmdlet aliases: `irm` (`Invoke-RestMethod`) for a clean output or `iwr` (`Invoke-WebRequest`) to inspect headers.*