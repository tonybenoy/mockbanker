# MockBanker

Generate and validate valid, checksum-correct **IBANs**, **Personal IDs**, **Credit Cards**, and more directly in your browser. No installation, no server — everything runs client-side via WebAssembly.

**[Try it live](https://tonybenoy.github.io/mockbanker/)**

## Features

- **IBAN Generation & Validation** — 96 countries supported, every code passes mod-97 checksum validation.
- **Personal ID Generation** — 31 formats including PESEL, personnummer, codice fiscale, JMBG, BSN, NIR, DNI, NIF, EGN, AMKA, and more.
- **Bank Accounts & SWIFT/BIC** — Generate test account numbers and routing codes for various countries.
- **Credit Cards** — Generate valid test card numbers (Visa, Mastercard, etc.) that pass Luhn checksum.
- **Company IDs** — Generate valid company registration numbers for supported countries.
- **Validator Tab** — (New) Validate IBANs, Personal IDs, Credit Cards, SWIFT codes, and Company IDs directly in the app.
- **PWA / Offline Support** — Install it on your device and use it without an internet connection.
- **Click-to-copy** — copy individual rows or all results at once.
- **Zero backend** — all logic runs in WASM, nothing leaves your browser.
- **Fast** — generates hundreds of valid codes in milliseconds.

## Tech Stack

- **[Leptos](https://leptos.dev/)** — reactive Rust web framework.
- **[idsmith](https://github.com/Sunyata-OU/idsmith)** — core generation & validation library (Rust).
- **[Trunk](https://trunkrs.dev/)** — WASM build tool.
- **GitHub Pages** — static hosting with automatic deploys.

## Development

```bash
# Prerequisites
rustup target add wasm32-unknown-unknown
cargo install trunk

# Dev server with hot reload
trunk serve --open

# Production build
trunk build --release
```

## License

MIT
