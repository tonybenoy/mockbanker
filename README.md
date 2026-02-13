# MockBanker

Generate valid, checksum-correct **IBANs** (96 countries) and **personal ID codes** (31 countries) directly in your browser. No installation, no server — everything runs client-side via WebAssembly.

**[Try it live](https://sunyata-ou.github.io/mockbanker/)**

## Features

- **96 IBAN countries** — every code passes mod-97 checksum validation
- **31 personal ID formats** — PESEL, personnummer, codice fiscale, JMBG, BSN, NIR, DNI, NIF, EGN, AMKA, and more
- **Click-to-copy** — copy individual rows or all results at once
- **Spaces toggle** — display IBANs with or without spaces
- **Gender & year filters** — narrow personal ID generation
- **Zero backend** — all logic runs in WASM, nothing leaves your browser
- **Fast** — generates hundreds of valid codes in milliseconds

## Tech Stack

- **[Leptos](https://leptos.dev/)** — reactive Rust web framework
- **[eu-test-data-generator](https://github.com/Sunyata-OU/EU-Test-Data-Generator)** — core generation library (Rust)
- **[Trunk](https://trunkrs.dev/)** — WASM build tool
- **GitHub Pages** — static hosting with automatic deploys

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

## Related Projects

| Project | Description |
|---------|-------------|
| [EU-Test-Data-Generator](https://github.com/Sunyata-OU/EU-Test-Data-Generator) | CLI tool & Rust library |
| [EU-Test-Data-Generator-GUI](https://github.com/Sunyata-OU/EU-Test-Data-Generator-GUI) | Native desktop app (egui) |

## License

MIT
