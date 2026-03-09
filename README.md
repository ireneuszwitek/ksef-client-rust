# KSeF Client Rust [![Package][package-img]][package-url] [![Documentation][documentation-img]][documentation-url]

[PL] komunikacja z API Krajowego Systemu e-Faktur (KSeF 2.0)

[EN] communication with the API of the digital platform for managing e-Invoicing in Poland (KSeF 2.0)

Biblioteka jest napisana w języku Rust; jest gotowa do integracji z istniejącymi projektami.

## 📜 Przykłady

Przykłady użycia biblioteki znajdują się w katalogu `examples`

- `get_access_token.rs` - autentykacja i pobranie access_token
- `refresh_token.rs` - odświeżenie wygasłego access_token
- `query_invoice_metadata.rs` - pobranie listy faktur ze stronicowaniem
- `export_invoice.rs` - eksport faktur używany do przyrostowego pobiernia faktur
- `get_invoice_qrcode.rs` - generowanie kodu QR dla ręcznie wskazanej faktury
- `get_invoice_qrcode_2.rs` - generowanie kodu QR dla pobranej faktury 
- `send_invoice_online.rs` - wysłanie faktury w trybie online
- `send_invoice_batch.rs` - wysłanie faktur w trybie wsadowym wraz z pobraniem UPO zbiorczego dla sesji i dla pierwszej wysłanej faktury
- `get_sessions.rs` - pobranie listy sesji


## 🔧 Instalacja

Dodaj do `Cargo.toml`:

```toml
[dependencies]
ksef = "0.7"
tokio = { version = "1.37", features = ["full"] }
chrono = { version = "0.4", features = ["serde"]} 
```
[package-img]: https://img.shields.io/crates/v/ksef.svg
[package-url]: https://crates.io/crates/ksef

[documentation-img]: https://docs.rs/ksef/badge.svg
[documentation-url]: https://docs.rs/ksef