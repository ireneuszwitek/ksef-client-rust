# KSeF Client Rust

[PL] komunikacja z API Krajowego Systemu e-Faktur (KSeF 2.0)

[EN] communication with the API of the digital platform for managing e-Invoicing in Poland (KSeF 2.0)

Biblioteka jest napisana w języku Rust; jest gotowa do integracji z istniejącymi projektami.

## 📜 Przykłady

Przykłady użycia biblioteki znajdują się w katalogu `examples`

- `get_access_token.rs` - autentykacja i pobranie access_token
- `refresh_token.rs` - odświeżenie wygasłego access_token
- `query_invoice_metadata.rs` - pobieranie listy faktur ze stronicowaniem
- `export_invoice.rs` - eksport faktur używany do przyrostowego pobiernia faktur
- `get_invoice_qrcode.rs` - generowanie kodu QR faktury
- `send_invoice_online.rs` - wysłanie faktury w trybie online
- `send_invoice_batch.rs` - wysłanie faktur w trybie wsadowym

## 🔧 Instalacja

Dodaj do `Cargo.toml`:

```toml
[dependencies]
ksef = "0.6"
tokio = { version = "1.37", features = ["full"] }
chrono = { version = "0.4", features = ["serde"]} 
```
