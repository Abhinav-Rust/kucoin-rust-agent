# KuCoin Automated Scalping Agent (Rust)

## Overview
An asynchronous, high-performance automated trading agent built in Rust to interface with the KuCoin Futures API. This project demonstrates secure, real-time order execution, dynamic payload parsing, and complex cryptographic authentication for centralized exchanges.

## Technical Stack
* **Language:** Rust
* **Async Runtime:** `tokio`
* **HTTP Client:** `reqwest`
* **Authentication:** HMAC-SHA256 signature generation

## Key Features
* **Secure API Integration:** Manages highly restrictive authenticated requests to KuCoin's V1 Futures endpoints.
* **Precision Order Routing:** Dynamically calculates position sizing, leverage locking, and target take-profit pricing based on live ticker data.
* **Asynchronous Execution:** Built to handle low-latency market entries and simultaneous limit-order exits.
* **Robust Error Handling:** Designed to navigate and resolve complex exchange-level risk checks, margin constraints, and strict JSON formatting requirements.

## Note on Usage
*This logic was built and battle-tested for live API environments. Note: KuCoin's internal terms of service prohibit the routing of promotional "Trial Funds" through third-party API executions.*
