<div align="center">

<img src="https://api.authvaultix.com/assets/img/logo.webp" alt="AuthVaultix Logo" width="80" height="80" />

# AuthVaultix Rust Example

**A complete, ready-to-use Rust CLI integration example for the [AuthVaultix](https://authvaultix.com) authentication platform.**

[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-CE422B?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![AuthVaultix](https://img.shields.io/badge/AuthVaultix-API%201.0-6366F1?style=for-the-badge)](https://authvaultix.com)
[![License](https://img.shields.io/badge/License-MIT-22c55e?style=for-the-badge)](LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20Us-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/muHy3qxcub)
[![Platform](https://img.shields.io/badge/Platform-Windows-0078D6?style=for-the-badge&logo=windows&logoColor=white)](https://www.microsoft.com/windows)

</div>

---

## 📖 Overview

This repository provides a **plug-and-play Rust CLI example** demonstrating how to integrate the [AuthVaultix](https://authvaultix.com) authentication API into your Rust application. It includes:

- 🔐 **Login** — Authenticate users with username & password
- 📝 **Register** — Create new accounts with a license key
- 🔑 **License Login** — Access the app using only a license key
- 🖥️ **HWID Detection** — Automatically reads the Windows User SID as hardware fingerprint
- 📊 **User Info Display** — Shows username, IP, HWID, creation date, last login & active subscriptions
- ⏱️ **Expiry Countdown** — Human-readable time-left format (`Xd Xh Xm`)

> Built as a **terminal-based interactive menu app** — fully blocking/synchronous using `reqwest` blocking client.

---

## 🗂️ Project Structure

```
authvaultix-rust-example/
├── Cargo.toml          # Dependencies & package metadata
├── run.bat             # Quick run script for Windows
└── src/
    ├── main.rs         # Entry point — interactive CLI menu
    └── lib.rs          # AuthVaultix API wrapper (core library)
```

---

## ⚡ Quick Start

### 1. Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable, **2024 edition**)
- Windows OS (HWID detection uses PowerShell / WMIC)
- An **AuthVaultix** account → [Register here](https://authvaultix.com)

### 2. Clone the Repository

```bash
git clone https://github.com/AuthVaultix-Rust-Example-main.git
cd AuthVaultix-Rust-Example-main
```

### 3. Configure Your Credentials

Open `src/main.rs` and fill in your application details from the [AuthVaultix Dashboard](https://authvaultix.com):

```rust
let mut AuthVaultixApp = AuthVaultix::new(
    "YourAppName",   // name
    "your_ownerid",  // ownerid
    "your_secret",   // secret
    "1.0"            // version
);
```

> ⚠️ **Never commit real credentials to a public repository.**

### 4. Build & Run

```bash
cargo run
```

Or use the included Windows batch script:

```batch
run.bat
```

---

## 🖥️ CLI Usage

When you run the app, you'll see an interactive menu:

```
Connecting...
✅ Initialized Successfully!

[1] Login
[2] Register
[3] License Login
[4] Exit
Choose option:
```

#### Login (Option 1)
```
Username: johndoe
Password: ••••••••

✅ Logged in!

=== User Data ===
Username: johndoe
IP: 192.168.x.x
HWID: S-1-5-21-...
Created: 2025-01-01 08:00:00 AM
Last Login: 2026-05-04 05:00:00 PM

Subscriptions:
[1] default | Expiry: 2027-01-01 12:00:00 AM | Timeleft: 240d 6h 0m
```

#### Register (Option 2)
```
Username: newuser
Password: ••••••••
License: XXXX-XXXX-XXXX-XXXX

✅ Registered Successfully!
```

#### License Login (Option 3)
```
License: XXXX-XXXX-XXXX-XXXX

✅ License Login Successful!
```

---

## 🧩 Library Usage (`lib.rs`)

You can use the `AuthVaultix` struct directly in your own Rust project:

### Initialize

```rust
use AuthVaultix_rust::AuthVaultix;

let mut app = AuthVaultix::new("AppName", "ownerid", "secret", "1.0");
app.init(); // Must be called before any other method
```

### Login

```rust
app.login("username", "password");
```

### Register

```rust
app.register("username", "password", "LICENSE-KEY");
```

### License Login

```rust
app.license_login("LICENSE-KEY");
```

---

## ⚙️ API Reference

### Authentication & Session
| Method | Returns | Description |
|---|---|---|
| `init()` | `()` | Initializes the session with the API. |
| `login(username, pass)` | `()` | Authenticates a user. |
| `register(username, pass, license, email)` | `()` | Registers a new user. |
| `license_login(license)` | `()` | Authenticates directly via license key. |
| `check()` | `bool` | Validates the current session. |
| `logout()` | `()` | Terminates session. |

### Account Management
| Method | Returns | Description |
|---|---|---|
| `upgrade(username, license)` | `bool` | Upgrades user's subscription. |
| `forgot_password(username, email)` | `bool` | Triggers a password reset email. |
| `change_username(new_username)` | `()` | Changes the current user's username. |

### Security & Logging
| Method | Returns | Description |
|---|---|---|
| `ban(reason)` | `bool` | Bans the currently authenticated user. |
| `check_blacklist()` | `bool` | Checks if the current HWID is blacklisted. |
| `log(message)` | `bool` | Sends a log message to the dashboard. |

### Variables & Data
| Method | Returns | Description |
|---|---|---|
| `get_global_var(varid)` | `Option<String>` | Fetches a global server variable. |
| `get_var(var_name)` | `Option<String>` | Fetches a user-specific variable. |
| `set_var(var_name, value)` | `bool` | Sets a user-specific variable. |
| `download(fileid)` | `Option<Vec<u8>>` | Securely downloads a file into a byte vector. |

### Communication
| Method | Returns | Description |
|---|---|---|
| `fetch_online()` | `Option<Vec<OnlineUser>>` | Retrieves a list of online clients. |
| `chat_send(message, channel)` | `bool` | Sends a chat message. |
| `chat_fetch(channel)` | `Option<Vec<ChatMessage>>` | Fetches chat history for a channel. |

---

## 📦 Dependencies

| Crate | Version | Purpose |
|---|---|---|
| [`reqwest`](https://crates.io/crates/reqwest) | 0.11 | HTTP client (blocking + JSON) |
| [`serde`](https://crates.io/crates/serde) | 1.0 | Serialization / Deserialization |
| [`serde_json`](https://crates.io/crates/serde_json) | 1.0 | JSON parsing |
| [`chrono`](https://crates.io/crates/chrono) | 0.4 | Unix timestamp → human-readable date |

`Cargo.toml`:
```toml
[dependencies]
chrono = "0.4"
reqwest = { version = "0.11", features = ["blocking", "json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

---

## 🖥️ HWID Detection

This example uses your **Windows User SID** as a hardware fingerprint. It tries two methods in order:

1. **PowerShell** (primary):
   ```powershell
   [System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
   ```

2. **WMIC** (fallback):
   ```cmd
   wmic useraccount where name='%USERNAME%' get sid /value
   ```

> If both fail, it falls back to `"UNKNOWN_HWID"`.

---

## 🔒 Security Notes

| Concern | Recommendation |
|---|---|
| Credentials in `main.rs` | Use environment variables or a config file in production |
| HWID Binding | AuthVaultix locks sessions to the detected SID by default |
| HTTPS | All API calls go to `https://authvaultix.com` — always encrypted |
| Error Handling | Production apps should handle panics gracefully with `Result<>` returns |

---

## 🛠️ Customization

- **Add 2FA support**: Extend the `login()` and `license_login()` payloads with a `"code"` field
- **Subscription gating**: Check `info.subscriptions` for a specific tier name before granting access
- **Cross-platform HWID**: Replace the PowerShell SID logic with a cross-platform crate like [`machine-uid`](https://crates.io/crates/machine-uid) for Linux/macOS support
- **GUI**: Integrate with [`egui`](https://crates.io/crates/egui) or [`tauri`](https://tauri.app/) to build a desktop GUI on top of this library

---

## 🤝 Contributing

Contributions, issues, and feature requests are welcome!

1. Fork the repository
2. Create a new branch: `git checkout -b feature/my-feature`
3. Commit your changes: `git commit -m 'Add my feature'`
4. Push to the branch: `git push origin feature/my-feature`
5. Open a Pull Request

---

## 💬 Support

- 📖 [AuthVaultix Documentation](https://authvaultix.com)
- 💬 [Discord Community](https://discord.gg/muHy3qxcub)
- 🐛 [Open an Issue](https://github.com/YOUR_USERNAME/authvaultix-rust-example/issues)

---

## 📄 License

This project is licensed under the **MIT License** — feel free to use, modify, and distribute it.

---

<div align="center">

Made with 🦀 Rust + ❤️ using [AuthVaultix](https://authvaultix.com)

</div>
