# Foundational TODOs (Before Complex Features)

- [ ] Implement `thiserror` in server and worker for structured error handling.
- [ ] Implement local setup also (essential for easy development).
- [ ] Implement Makefile/process-compose & Docker/Podman for automating local setup.
- [ ] **Configuration Management:** Implement loading config from file (`config.toml`) and env vars (e.g., using `figment`). ⚙️
- [ ] **Job Definition Parsing:** Implement basic parsing for job steps (commands).
- [ ] **Basic Job Execution:** Implement core worker logic to run commands (`tokio::process::Command`) and capture output/status.
- [ ] **Job Status Reporting (Worker -> Server):** Send `JobResult` via gRPC.
- [ ] **Database: Job State Machine:** Define and use statuses (`pending`, `running`, `success`, `failed`) in DB. 🔄
- [ ] **Basic Agent Scheduling:** Server logic to find an `online` worker and assign a `pending` job.
- [ ] **Implement `StatsUpdate` WebSocket Event:** Broadcast online/offline counts.
- [ ] **Cargo Workspace:** Structure project as a Cargo workspace (server + worker). ✅
- [ ] **Rename:** agent -> worker.
- [ ] **Logging:** Implement proper logging in server and worker (e.g., using `tracing`). 🪵
- [ ] **Code Formatting:** Set up and enforce `cargo fmt`.
- [ ] **CI:** Implement GitHub Actions flow for basic checks (formatting, build).
- [ ] **Graceful Shutdown:** Implement signal handling (Ctrl+C) for clean shutdown (`tokio::signal`). 🛑
- [ ] **Frontend Action:** Implement first frontend button to start a job (requires REST endpoint).
- [ ] **Deployment Config:** Differentiate between dev and prod configurations/startup.
- [ ] **Documentation:** Explore auto-generating docs with `cargo doc`. 📄
- [ ] **Refactor:** Apply learned Rust project structuring principles.

# Features After Creating Good Enough Base

- [ ] Proper frontend setup with React, TypeScript, MobX, TanStack Router.
- [ ] **Real-time Log Streaming:** Worker -> Server (gRPC) -> Frontend (WebSocket).
- [ ] **Code Checkout:** Worker logic to clone repo and checkout commit (e.g., `git2` crate).
- [ ] Frontend CI: Storybook, Vitest, Linting, etc.
- [ ] Authentication/Authorization.
- [ ] More Sophisticated Scheduling (capabilities, limits).
- [ ] Job Cancellation.
- [ ] Artifacts/Caching.