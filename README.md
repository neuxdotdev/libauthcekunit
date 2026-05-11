# libauthcekunit

> **Super robust, secure, and RFC‑compliant CSRF token extractor & authentication library for Rust – with configurable retry, structured logging, full cookie handling, and C FFI support.**

<p align="center">
  <a href="https://crates.io/crates/libauthcekunit"><img src="https://img.shields.io/crates/v/libauthcekunit.svg?style=flat-square&logo=rust" alt="crates.io version"></a>
  <a href="https://docs.rs/libauthcekunit"><img src="https://img.shields.io/docsrs/libauthcekunit?style=flat-square&logo=docsdotrs" alt="docs.rs"></a>
  <a href="https://github.com/yourusername/libauthcekunit/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/libauthcekunit?style=flat-square" alt="license"></a>
  <a href="https://github.com/yourusername/libauthcekunit/actions"><img src="https://img.shields.io/github/actions/workflow/status/yourusername/libauthcekunit/ci.yml?branch=main&style=flat-square&logo=github" alt="build"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/rust-1.70%2B-blue?style=flat-square&logo=rust" alt="Rust version"></a>
  <a href="https://crates.io/crates/reqwest"><img src="https://img.shields.io/badge/http-reqwest-orange?style=flat-square" alt="reqwest"></a>
  <a href="https://github.com/yourusername/libauthcekunit"><img src="https://img.shields.io/badge/C_FFI-available-success?style=flat-square&logo=c" alt="C FFI"></a>
  <a href="https://www.conventionalcommits.org"><img src="https://img.shields.io/badge/conventional%20commits-1.0.0-yellow.svg?style=flat-square" alt="Conventional Commits"></a>
</p>

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
  - [Rust library usage](#rust-library-usage)
  - [C FFI usage](#c-ffi-usage)
- [Environment Variables Configuration](#environment-variables-configuration)
  - [Complete list of env vars](#complete-list-of-env-vars)
  - [Example `.env` file](#example-env-file)
- [Library API Reference](#library-api-reference)
  - [Module `auth`](#module-auth)
  - [Module `cookies`](#module-cookies)
  - [Module `client`](#module-client)
  - [Module `parser`](#module-parser)
  - [Module `config`](#module-config)
  - [Module `error`](#module-error)
  - [Module `types`](#module-types)
  - [Module `logging`](#module-logging)
  - [High‑level functions](#highlevel-functions)
- [Retry & Resilience Architecture](#retry--resilience-architecture)
- [Cookie Handling (RFC 6265)](#cookie-handling-rfc-6265)
- [CSRF Token Extraction](#csrf-token-extraction)
- [Logging & Debugging](#logging--debugging)
- [C Foreign Function Interface (FFI)](#c-foreign-function-interface-ffi)
  - [Header generation](#header-generation)
  - [Function list](#function-list)
  - [Memory model & ownership](#memory-model--ownership)
- [Makefile Usage](#makefile-usage)
- [Project Structure](#project-structure)
- [Development & Contributing](#development--contributing)
- [FAQ](#faq)
- [License](#license)

---

## Overview

`libauthcekunit` is a Rust library that automates the process of logging into a web application that uses CSRF protection (like Laravel), extracting the CSRF token, managing cookies strictly according to [RFC 6265](https://tools.ietf.org/html/rfc6265), and exposing the functionality both as a high‑level Rust API and a C‑compatible FFI.

It is built for **robustness** and **security**:

- All HTTP requests are retried with exponential backoff on transient errors.
- Cookies are parsed, stored, and matched following RFC 6265 (domain, path, expiration, secure, same‑site).
- CSRF tokens are validated against configurable length and character‑set rules.
- No sensitive data (tokens, passwords) ever appears in log output.
- After login, the session is **verified** by accessing a protected page (configurable).

The library provides a `CookieJar` that can be serialised to/from JSON, making it easy to persist sessions across program restarts.

---

## Features

- [x] **CSRF‑aware login/logout** – automatically handles `_token` extraction and submission.
- [x] **Strict cookie handling** – RFC 6265 domain/path matching, expiration, secure flag, `SameSite`.
- [x] **Exponential backoff retry** – for network errors and server errors (5xx).
- [x] **Configurable via environment variables** – timeouts, retries, user agent, credentials, etc.
- [x] **Structured logging** using `tracing` – log level controlled by `RUST_LOG`.
- [x] **Session verification** after login – checks for a configurable string on a protected page.
- [x] **C FFI bindings** – usable from C, C++, Python ctypes, or any language with C interop.
- [x] **Serializable `CookieJar`** – save/load sessions to JSON files.
- [x] **Multi‑platform** – Linux, macOS, Windows (with proper TLS backend). -[x] **Zero‑cost abstractions** – efficient, no unsafe code except minimal FFI glue.

---

## Installation

### For Rust projects

Add this to your `Cargo.toml`:

```toml
[dependencies]
libauthcekunit = "2.0.0"
```

By default, it uses **rustls** for TLS. If you prefer **native‑tls** (OpenSSL), enable the feature:

```toml
libauthcekunit = { version = "2.0.0", default-features = false, features = ["native-tls"] }
```

### For C projects

Clone the repository and build:

```bash
git clone https://github.com/yourusername/libauthcekunit.git
cd libauthcekunit
make all        # builds static and shared library, generates C header
sudo make install   # installs to /usr/local by default
```

The header `libauthcekunit.h` and the libraries will be placed in `$PREFIX/include` and `$PREFIX/lib`.

---

## Quick Start

### Rust library usage

```rust
use libauthcekunit::{login, logout, fetch_token, CookieJar, init_logging};

// (Optional) initialise logging once
init_logging();

// Set required environment variables, or use .env file
let base_url = "https://example.com";

// Login – returns a CookieJar with the session
let jar: CookieJar = login(base_url).expect("Login failed");

// The jar can be used for further authenticated requests
// ... use jar with `RobustHttpClient` or your own HTTP client

// Save session to file for later reuse
jar.save_to_file("session.json").unwrap();

// Later, load the jar back
let jar = CookieJar::load_from_file("session.json").unwrap();

// Logout (destroys server session, but returns updated jar)
let final_jar = logout(base_url, &jar).expect("Logout failed");
```

If you only need the CSRF token from a page (no login):

```rust
let token = libauthcekunit::fetch_token("https://example.com/login").unwrap();
println!("CSRF token: {}", token);
```

### C FFI usage

```c
#include "libauthcekunit.h"
#include <stdio.h>

int main() {
    libauthcekunit_init_logging();

    // Login
    CookieJarHandle* jar = libauthcekunit_login("https://example.com");
    if (jar == NULL) {
        fprintf(stderr, "Login failed\n");
        return 1;
    }

    // Fetch a CSRF token from a page
    char token[256];
    if (libauthcekunit_fetch_token("https://example.com/form", token, sizeof(token))) {
        printf("Token: %s\n", token);
    }

    // Extract token from raw HTML
    const char* html = "<input name=\"_token\" value=\"abc123\">";
    char extracted[64];
    if (libauthcekunit_extract_token(html, extracted, sizeof(extracted))) {
        printf("Extracted: %s\n", extracted);
    }

    // Logout (consumes jar handle)
    int result = libauthcekunit_logout(jar, "https://example.com");
    // jar is now invalid; do not use it again

    // Or free the jar without logout
    // libauthcekunit_free_jar(jar);

    return 0;
}
```

Compile & link:

```bash
gcc -o myapp myapp.c -L/usr/local/lib -lauthcekunit -lpthread -ldl -lm
```

---

## Environment Variables Configuration

All configuration is done via environment variables (or a `.env` file loaded automatically by `dotenv`).  
Prefix: `LIB_CEKUNIT_AUTH_ENV_`.

### Complete list of env vars

| Variable                                 | Type     | Default                          | Description                                                                                                               |
| ---------------------------------------- | -------- | -------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| `LIB_CEKUNIT_AUTH_ENV_TIMEOUT_SECS`      | `u64`    | `30`                             | Per‑request timeout (seconds).                                                                                            |
| `LIB_CEKUNIT_AUTH_ENV_MAX_RETRIES`       | `u32`    | `3`                              | Maximum number of retry attempts (including the first).                                                                   |
| `LIB_CEKUNIT_AUTH_ENV_RETRY_INITIAL_MS`  | `u64`    | `500`                            | Initial backoff delay in milliseconds.                                                                                    |
| `LIB_CEKUNIT_AUTH_ENV_MAX_REDIRECTS`     | `u32`    | `5`                              | Maximum redirects to follow.                                                                                              |
| `LIB_CEKUNIT_AUTH_ENV_FOLLOW_REDIRECTS`  | `bool`   | `true`                           | Whether to follow redirects.                                                                                              |
| `LIB_CEKUNIT_AUTH_ENV_TOKEN_MIN_LEN`     | `usize`  | `10`                             | Minimum allowed CSRF token length.                                                                                        |
| `LIB_CEKUNIT_AUTH_ENV_TOKEN_MAX_LEN`     | `usize`  | `1024`                           | Maximum allowed CSRF token length.                                                                                        |
| `RUST_LOG`                               | `string` | `"info"`                         | Log level (trace, debug, info, warn, error).                                                                              |
| `LIB_CEKUNIT_AUTH_ENV_USER_AGENT`        | `string` | `"LIB_CEKUNIT_AUTH_ENVUnit/2.0"` | User‑Agent header for HTTP requests.                                                                                      |
| `LIB_CEKUNIT_AUTH_ENV_EMAIL`             | `string` | `""`                             | Email/username for login.                                                                                                 |
| `LIB_CEKUNIT_AUTH_ENV_PASSWORD`          | `string` | `""`                             | Password for login.                                                                                                       |
| `LIB_CEKUNIT_AUTH_ENV_LOGIN_VERIFY_URL`  | `string` | `"/dashboard"`                   | Relative URL of a protected page used to verify login success.                                                            |
| `LIB_CEKUNIT_AUTH_ENV_LOGIN_VERIFY_TEXT` | `string` | `""`                             | (Optional) Text that must appear on the verification page. If empty, verification just checks that the page is reachable. |
| `LIB_CEKUNIT_AUTH_ENV_ENV_PATH`          | `string` | –                                | Path to a `.env` file (if not in current directory).                                                                      |

### Example `.env` file

```ini
LIB_CEKUNIT_AUTH_ENV_EMAIL=admin@example.com
LIB_CEKUNIT_AUTH_ENV_PASSWORD=secret
LIB_CEKUNIT_AUTH_ENV_TIMEOUT_SECS=60
LIB_CEKUNIT_AUTH_ENV_MAX_RETRIES=5
RUST_LOG=libauthcekunit=debug
```

---

## Library API Reference

### Module `auth`

#### `pub fn login(base_url: &str) -> Result<CookieJar>`

Performs login:

1. Fetches the login page, extracts the CSRF token.
2. Submits the login form using credentials from environment variables.
3. **Verifies** the session by accessing the configured `login_verify_url`.
4. Returns the session `CookieJar`.

Returns `Err(AuthError::Config(...))` if email/password are missing, `Err(AuthError::LoginFailed(...))` if the verification step fails.

#### `pub fn logout(base_url: &str, cookie_jar: &CookieJar) -> Result<CookieJar>`

Logs out by fetching the dashboard (to get a CSRF token), then posting to `/logout`. Returns the final cookie jar (which typically has the session cookie cleared).

### Module `cookies`

#### `pub struct Cookie`

A parsed cookie with all RFC attributes.

| Field       | Type                 | Description                                      |
| ----------- | -------------------- | ------------------------------------------------ |
| `name`      | `String`             | Cookie name.                                     |
| `value`     | `String`             | Cookie value.                                    |
| `domain`    | `Option<String>`     | Domain scope (defaults to request host).         |
| `path`      | `Option<String>`     | Path scope (defaults to request path directory). |
| `expires`   | `Option<SystemTime>` | Expiration time (UTC).                           |
| `secure`    | `bool`               | Send only over HTTPS.                            |
| `http_only` | `bool`               | Not accessible via JavaScript.                   |
| `same_site` | `Option<String>`     | `Strict`, `Lax`, or `None`.                      |

**Methods:**

- `from_set_cookie(header: &str, request_url: &Url) -> Option<Cookie>` – parses a `Set‑Cookie` header, returns `None` if expired or malformed.
- `matches(&self, url: &Url) -> bool` – tests if this cookie should be sent to the given URL (domain, path, secure, expiry).
- `to_header(&self) -> String` – returns `name=value` for a `Cookie` header.

#### `pub struct CookieJar`

A collection of cookies, stored by name (one per name).

**Methods:**

- `new() -> Self`
- `add_from_set_cookie(&mut self, header: &str, url: &Url)` – parses and inserts a cookie.
- `get_cookies_for_url(&self, url: &Url) -> Vec<&Cookie>` – returns cookies that match the URL.
- `cookie_header(&self, url: &Url) -> String` – builds the `Cookie` header value.
- `load_from_file(path: &str) -> io::Result<Self>` – loads from JSON, discarding expired cookies.
- `save_to_file(&self, path: &str) -> io::Result<()>` – serialises to JSON.
- `clear_expired(&mut self)` – removes all expired cookies.
- `len(&self) -> usize`, `is_empty(&self) -> bool`

_Note:_ The jar stores only the **most recent** cookie for a given name. This is sufficient for typical session handling but does not support multiple cookies with the same name from different domains/paths.

### Module `client`

#### `pub struct RobustHttpClient`

A synchronous HTTP client built on `reqwest`, with automatic retry and cookie handling.

**Methods:**

- `new() -> Result<Self>` – creates client using global `CONFIG`.
- `get(&self, url: &str, cookies: &CookieJar) -> Result<(String, CookieJar)>` – performs GET, returns body and updated jar.
- `post_form(&self, url: &str, params: &[(&str, &str)], cookies: &CookieJar) -> Result<(String, CookieJar)>` – performs POST with `application/x-www-form-urlencoded` body.

Both methods implement retry with exponential backoff on server errors (5xx) and network failures. Client errors (4xx) are not retried.

### Module `parser`

#### `pub fn extract_token(html: &str) -> Result<String>`

Extracts a CSRF token from HTML. It first tries a regex pattern searching for `name="_token" value="..."`, then falls back to a manual scan. The token is validated for length and allowed characters.

### Module `config`

#### `pub static CONFIG: Lazy<Config>`

Global configuration object, initialised once from environment variables.

#### `pub struct Config`

Fields (all public):

- `request_timeout_secs: u64`
- `max_retries: u32`
- `retry_initial_interval_ms: u64`
- `max_redirects: u32`
- `follow_redirects: bool`
- `token_min_length: usize`
- `token_max_length: usize`
- `log_level: String`
- `user_agent: String`
- `email: String`
- `password: String`
- `login_verify_url: String`
- `login_verify_text: String`

### Module `error`

#### `pub enum AuthError`

| Variant                     | Description                               |
| --------------------------- | ----------------------------------------- |
| `Http(reqwest::Error)`      | Underlying HTTP client error.             |
| `UrlParse(url::ParseError)` | URL parse error.                          |
| `TokenNotFound`             | CSRF token not found in HTML.             |
| `TokenInvalid { reason }`   | Token fails validation (length, charset). |
| `Config(String)`            | Configuration error.                      |
| `Io(std::io::Error)`        | Filesystem I/O error.                     |
| `Regex(regex::Error)`       | Regular expression compilation error.     |
| `MaxRetriesExceeded`        | All retry attempts exhausted.             |
| `HttpStatus(u16)`           | Non‑success HTTP status (4xx).            |
| `LoginFailed(String)`       | Session verification after login failed.  |

`pub type Result<T> = std::result::Result<T, AuthError>;`

### Module `types`

#### `pub struct CsrfToken`

- `value: String`
- `source_url: String`
- `extracted_at: SystemTime`

#### `pub struct FetchOptions`

(Currently not used by high‑level functions, but available for future use or custom client configuration.)

- `timeout_secs: u64`
- `max_retries: u32`
- `retry_initial_interval_ms: u64`
- `follow_redirects: bool`
- `max_redirects: u32`

### Module `logging`

#### `pub fn init_logging()`

Initialises the `tracing` subscriber (once). Uses the `RUST_LOG` environment variable.

---

## High‑level functions

These are convenience wrappers exposed directly in `lib.rs`:

#### `pub fn fetch_token_and_cookies(url: &str) -> Result<(String, CookieJar)>`

Fetches a page, extracts the CSRF token, and returns the token together with all `Set‑Cookie` cookies in a new `CookieJar`.

#### `pub fn fetch_token(url: &str) -> Result<String>`

Like above, but returns only the token.

#### `pub fn fetch_cookies(url: &str) -> Result<CookieJar>`

Like above, but returns only the jar.

> **Security note:** The token content is **not** printed to logs; only its length is recorded in debug mode.

---

## Retry & Resilience Architecture

The `RobustHttpClient` employs **exponential backoff** for transient failures:

- Initial backoff: `CONFIG.retry_initial_interval_ms` (default 500 ms)
- Max interval: 5 s
- Max total elapsed time: `request_timeout_secs * max_retries`
- Retries are triggered on: network errors (connection refused, timeouts, etc.) and server errors (5xx responses).
- Client errors (4xx) are immediately returned as `AuthError::HttpStatus`.
- If all retries fail, `AuthError::MaxRetriesExceeded` is returned.

The backoff parameters are all configurable via environment variables.

---

## Cookie Handling (RFC 6265)

The cookie module implements **RFC 6265** strictly, ensuring secure and correct session management.

- **Domain matching:** `example.com` matches `example.com` and `*.example.com`; a leading dot is stripped on input.
- **Path matching:** `/foo` matches `/foo`, `/foo/`, `/foo/bar`, but **not** `/foobar`.
- **Expiration:** Cookies are rejected at parse time if already expired, and again at every `matches()` call. `clear_expired()` removes them from the jar.
- **Secure flag:** Cookies marked `Secure` are only sent over HTTPS.
- **SameSite:** Parsed and stored, but not enforced by the library (it assumes the caller respects it).
- **Serialization:** Cookie jars are serialized to JSON, and expired cookies are purged automatically when loaded.

The implementation has been **audited** and covers edge cases like default path derivation from the request URL.

---

## CSRF Token Extraction

The parser searches for the standard Laravel pattern:

```html
<input type="hidden" name="_token" value="randomtoken" />
```

It uses a regex designed to handle both `'` and `"` quotes, and falls back to a manual scan if the regex fails. The extracted token is then validated:

- Length must be between `token_min_length` and `token_max_length` (configurable).
- Characters must be alphanumeric, or one of `_`, `-`, `+`, `=`, `/` (base64url characters).
- **No token content is ever printed in logs** – only the token length appears in debug logs.

---

## Logging & Debugging

The library uses the `tracing` ecosystem. Log output is controlled by the `RUST_LOG` environment variable. Example:

```bash
RUST_LOG=libauthcekunit=debug cargo run
```

Important debug events include:

- HTTP request/response summaries (status, body length)
- Cookie additions and expirations
- Retry attempts and backoff durations
- Token extraction steps (regex hit, manual fallback)

All sensitive data (tokens, credentials) is omitted from log lines.

---

## C Foreign Function Interface (FFI)

The library can be compiled as a **C dynamic/shared library** (`cdylib`) and a **static library** (`staticlib`). All functions are marked `#[no_mangle]` and `extern "C"`, and use only C‑compatible types.

### Header generation

A C header can be generated using `cbindgen`:

```bash
make header   # requires cbindgen installed
```

### Function list

| Function                                                                            | Description                                                                                 |
| ----------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| `void libauthcekunit_init_logging(void)`                                            | Initialise logging (needed only once).                                                      |
| `CookieJarHandle* libauthcekunit_login(const char* base_url)`                       | Login, returns a handle to the session jar (or `NULL` on error).                            |
| `int libauthcekunit_logout(CookieJarHandle* handle, const char* base_url)`          | Logout, **consumes the handle** (do not use it afterwards). Returns 0 on success.           |
| `void libauthcekunit_free_jar(CookieJarHandle* handle)`                             | Free a jar handle without logging out.                                                      |
| `size_t libauthcekunit_fetch_token(const char* url, char* out, size_t out_size)`    | Fetch token, writes to `out` including null terminator; returns bytes written (0 on error). |
| `int libauthcekunit_fetch_cookies(const char* url, const char* file_path)`          | Fetch cookies and save directly to a JSON file.                                             |
| `size_t libauthcekunit_extract_token(const char* html, char* out, size_t out_size)` | Extract token from HTML string.                                                             |
| `const char* libauthcekunit_get_email(void)`                                        | Returns pointer to configured email (static lifetime).                                      |
| `const char* libauthcekunit_get_user_agent(void)`                                   | Returns pointer to configured user agent (static lifetime).                                 |

### Memory model & ownership

- `libauthcekunit_login` returns a `CookieJarHandle*` that must be either passed to `libauthcekunit_logout` (which frees it) or explicitly freed with `libauthcekunit_free_jar`.
- `libauthcekunit_logout` consumes the handle – after this call the pointer becomes invalid.
- Strings returned by `get_email` / `get_user_agent` point to static memory and must not be freed.

---

## Makefile Usage

The included `Makefile` provides common tasks:

```bash
make all              # Build static, shared, header
make static           # Build static library (.a)
make dynamic          # Build shared library (.so)
make header           # Generate C header
make test             # Build and run static test (loads .env)
make test-dyn         # Build and run dynamic test
make install          # Install to /usr/local (prefix can be changed)
make uninstall        # Remove installed files
make clean            # Clean build artifacts
```

Set `INSTALL_PREFIX` to change installation path:

```bash
make install INSTALL_PREFIX=/usr
```

---

## Project Structure

```
libauthcekunit/
├── Cargo.toml
├── Cargo.lock
├── Makefile
├── LICENSE
├── README.md
└── src/
    ├── lib.rs          # Public API and re-exports
    ├── auth.rs         # Login / logout logic
    ├── client.rs       # Robust HTTP client with retry
    ├── config.rs       # Configuration from environment
    ├── cookies.rs      # Cookie and CookieJar (RFC 6265)
    ├── error.rs        # Error types
    ├── ffi.rs          # C bindings
    ├── logging.rs      # Tracing initialisation
    ├── parser.rs       # CSRF token extraction
    └── types.rs        # Shared types
```

---

## Development & Contributing

1. **Clone the repository:**

   ```bash
   git clone https://github.com/yourusername/libauthcekunit.git
   cd libauthcekunit
   ```

2. **Build and test:**

   ```bash
   cargo build
   cargo test
   ```

3. **Run the test suite with environment variables:**

   ```bash
   cp .env.example .env   # edit .env with test credentials
   cargo test
   ```

4. **Build C libraries and header:**

   ```bash
   make all
   ```

5. **Linting and formatting:**
   ```bash
   cargo clippy --all-targets --all-features
   cargo fmt --all -- --check
   ```

### Contribution Guidelines

- Follow [Conventional Commits](https://www.conventionalcommits.org/).
- Add tests for new features.
- Ensure `cargo test` passes and `cargo clippy` is clean.
- For changes to the FFI, update the header with `make header` and test the C example.

---

## FAQ

**Q: Why build a custom cookie jar instead of using `reqwest`'s built‑in one?**  
A: `reqwest`'s cookie store is tied to its async runtime and not as strict about RFC 6265 validation. Our jar gives us full control over parsing, matching, and serialisation, plus it works seamlessly with the C FFI.

**Q: Can I use this library with async Rust?**  
A: The current API is synchronous (blocking). For async use, you would need to wrap the calls with `tokio::task::spawn_blocking` or use an async version of the client. An async feature may be added in the future.

**Q: How do I handle websites that use a different CSRF token field name?**  
A: The parser specifically looks for `name="_token"`. You can customise the regex in `parser.rs` or use `extract_token` on raw HTML after a custom extraction step.

**Q: Is the library thread‑safe?**  
A: Yes. All public types are `Send + Sync`, and the `CONFIG` is initialised once via `Lazy`.

**Q: The login verifier fails, but I know the credentials are correct. Why?**  
A: Set `LIB_CEKUNIT_AUTH_ENV_LOGIN_VERIFY_TEXT` to the empty string to disable text matching. Some sites redirect after login, and the verification URL might not contain the expected text. Adjust `LOGIN_VERIFY_URL` to point to a static page (e.g., `/profile`).

---

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.

---

<p align="center">
  Built with ❤️ in Rust • RFC 6265 compliant • Battle‑tested retry logic • Safe C FFI
</p>
```
