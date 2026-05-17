# 🪐 ContextFlow: Local-First Developer Hyper-Context Engine

<p align="center">
  <img src="./vscode-extension/icon.png" width="128" height="128" alt="ContextFlow Constellation Icon" />
</p>

> **[English](./README.md) | [한국어](./README.ko.md)**

ContextFlow is a high-performance, local-first developer context engine that seamlessly captures file modifications, Git diffs, terminal execution sequences, and command exit statuses into an encrypted-ready local SQLite database. It leverages this real-time session memory to generate instant activity summaries, engineering plans, and automated self-healing code patches (**Agentic Fixes**) based on contextual database timelines.

All background tracking and privacy filtering are powered by an ultra-fast, native **Rust daemon (`daemon-rust`)** to maintain a near-zero system footprint. It integrates natively with VS Code via an official extension, giving developers instant access to rich AI context right inside their IDE.

---

## 📖 Table of Contents
1. [Core Features & Architecture](#1-core-features--architecture)
2. [Setup Guide](#2-setup-guide)
3. [Daemon & Shell Hook Setup](#3-daemon--shell-hook-setup)
4. [CLI Command Usage](#4-cli-command-usage)
5. [Real-World Developer Scenarios (Examples)](#5-real-world-developer-scenarios-examples)
6. [Outbound Privacy & Safety Shield (RAG Privacy)](#6-outbound-privacy--safety-shield-rag-privacy)

---

## 1. Core Features & Architecture

* **🚀 Ultra-lightweight Native Engine**: The entire daemon is engineered as a compiled **Rust native binary**, keeping background resource usage under 10MB RAM and less than 1% CPU for absolute stealth execution.
* **⚡ Asynchronous Multithreaded Engine**: Powered by the `Tokio` async runtime, driving concurrent lightweight `Axum` HTTP routers alongside native OS file change watchers (`Notify`).
* **📦 Concurrency-Isolated Local Database**: All system timelines are recorded into a local `memory.db` file. Synchronous SQLite writes are safely isolated within `tokio::task::spawn_blocking` pools to prevent thread starvation.
* **🛡️ Privacy-First Prompt Redactor**: Prior to outbound LLM delivery, our local compiled-regex `PrivacyFilter` automatically scans and masks secrets (API keys, JWT bearer tokens, private keys) to guarantee zero leakage.

---

## 2. Setup Guide

### 2.1 API Credentials
Set the API key of your preferred AI provider (Gemini is native; OpenAI is supported as fallback) in your environment variables or a local `.env` file at the project root.

```env
# .env File Template
GEMINI_API_KEY="your_real_gemini_key_here"
OPENAI_API_KEY="your_openai_api_key_here"
```

### 2.2 Configuration (`.contextflow.json`)
You can configure custom ports or model settings by creating a `.contextflow.json` file in your workspace root.

```json
{
  "provider": "gemini",
  "model": "gemini-2.5-flash",
  "baseUrl": "https://generativelanguage.googleapis.com",
  "port": 49152
}
```

---

## 3. Daemon & Shell Hook Setup

The ContextFlow watcher daemon must always be launched from the **root directory of your project**.

### 3.1 Launching the Daemon
Run the compiled Rust binary in the background or in a separate terminal panel:
```bash
# Navigate to the project root and start the daemon
cd ContextFlow
./daemon-rust/target/release/daemon-rust start
```

### 3.2 Shell Hook Registration
To automatically capture terminal commands and their exit statuses, add the following hook configuration to your shell's startup profile (e.g., `~/.zshrc` or `~/.bashrc`):

#### For Zsh Users (Add to the end of `~/.zshrc`):
```zsh
# ContextFlow Shell Event Hook (Set CF_PATH to your actual workspace folder)
CF_PATH="/path/to/ContextFlow"

chpwd_contextflow() {
  # Notify daemon on directory changes
  bash "$CF_PATH/scripts/shell-hook.sh" "dir_change" "$PWD" 0
}
precmd_contextflow() {
  local exit_status=$?
  if [ -n "$LAST_CMD" ]; then
    # Ship command string and exit code to local SQLite
    bash "$CF_PATH/scripts/shell-hook.sh" "terminal_command" "$LAST_CMD" "$exit_status"
    unset LAST_CMD
  fi
}
preexec_contextflow() {
  # Temporarily store executing command
  LAST_CMD="$1"
}
add-zsh-hook chpwd chpwd_contextflow
add-zsh-hook precmd precmd_contextflow
add-zsh-hook preexec preexec_contextflow
```

---

## 4. CLI Command Usage

Once the daemon is active, manage and inspect the local workspace intelligence session using the following CLI commands:

### 4.1 Check Daemon Status
```bash
./daemon-rust/target/release/daemon-rust status
```
* **Output Example**:
  ```text
  --- ContextFlow Daemon Status ---
  Status: Active (listening on localhost)
  Active Session: cf_lru_cache_refactoring
  Total Stored Events: 142
  Database Path: .contextflow\memory.db
  ---------------------------------
  ```

### 4.2 Dynamic Configuration Modification
```bash
# Shift active provider to OpenAI
./daemon-rust/target/release/daemon-rust config --set provider=openai

# Query the active provider model
./daemon-rust/target/release/daemon-rust config --get model
```

### 4.3 Real-Time Session Summarization & Planning
* **Summarize**: Generate a high-fidelity technical summary of your recent 30 file and console actions:
  ```bash
  ./daemon-rust/target/release/daemon-rust summarize
  ```
* **Plan**: Establish a step-by-step engineering plan for your next development iteration:
  ```bash
  ./daemon-rust/target/release/daemon-rust plan
  ```

### 4.4 Automated Self-Healing Patch (Agentic Fix)
Scan the last failed terminal command (Exit Code > 0) or source code files and generate a drop-in code correction:
```bash
./daemon-rust/target/release/daemon-rust fix
```

---

## 5. Real-World Developer Scenarios (Examples)

### 💡 Scenario 1: Self-Healing Compile Errors (Agentic Fix)
A developer runs a compiler command (`cargo build`) that fails with exit code `1`:
1. The terminal shell hook immediately intercepts the exit code `1` and relays the `terminal_command` event to the local sqlite database.
2. The developer executes `./daemon-rust/target/release/daemon-rust fix` (or invokes `Fix with AI` from the VS Code command palette).
3. ContextFlow retrieves the recent compile error log and the target source file (`src/main.rs`).
4. AI calculates a precise, low-overhead **SEARCH/REPLACE correction patch**:
   ```text
   🤖 Requesting agentic fix recommendation...
   
   --- Recommended Fix ---
   The compiler error was caused by a moved String value on line 321. 
   Add a `.clone()` call to preserve ownership:
   
   <<<<<<< SEARCH
   let value = config.provider;
   println!("Provider: {}", value);
   =======
   let value = config.provider.clone();
   println!("Provider: {}", value);
   >>>>>>> REPLACE
   ```

### 💡 Scenario 2: Perfect Session Continuation (Session Resume)
Returning to a feature branch after a weekend or context-switching from an emergency hotfix:
1. Re-activate your session: `cf_main_feature`.
2. Run `./daemon-rust/target/release/daemon-rust summarize` to let AI read the chronological SQLite diff records and reconstruct the developer intent:
   ```text
   📝 Analyzing real-time context (Events: 85)
   
   You were last working on the 'LRU Cache concurrency test harness' inside 'src/db.rs'. 
   The final recorded file change log shows the mock assertions are fully updated, but the test suite has not yet been run.
   Action Suggestion: Execute 'cargo test' to verify cache integrity.
   ```

---

## 6. Outbound Privacy & Safety Shield (RAG Privacy)

ContextFlow is engineered from the ground up for high-security commercial workspaces. It uses a native, high-performance **Privacy Sandbox** to prevent sensitive credentials from leaking to public cloud API providers.

Before any local event context (source modifications, shell outputs) is transmitted outbound to a cloud model, it must pass through the `PrivacyFilter` ([privacy.rs](./daemon-rust/src/intelligence/privacy.rs)) pipeline.

* **API Keys & Credentials**: Redacts API keys, raw secrets, and password definitions matching `(?i)(api[_-]?key|secret|password|token)\s*[:=]\s*...` patterns, replacing them with a secure `[REDACTED_SECRET]` tag.
* **Bearer Tokens**: Scans JWT headers or HTTP curl commands, converting tokens into `Bearer [REDACTED_BEARER]` placeholders.
* **Private SSH Keys**: Detects raw private cryptographic key blocks (`-----BEGIN RSA PRIVATE KEY-----`) in terminal traces or files and strips them globally into `[REDACTED_PRIVATE_KEY]`.

---
© 2026 Trisoft. All rights reserved. (Docs folder excluded for draft isolation.)
