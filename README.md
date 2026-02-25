# KSeF Client Rust

[PL] komunikacja z API Krajowego Systemu e-Faktur (KSeF 2.0)

[EN] communication with the API of the digital platform for managing e-Invoicing in Poland (KSeF 2.0)

Biblioteka jest napisana w języku Rust; jest gotowa do integracji z istniejącymi projektami.

## 📜 Przykłady

Przykłady użycia biblioteki znajdują się w katalogu `examples`

- `get_access_token.rs` - autentykacja i pobranie access_token
- `refresh_token.rs` - odświeżenie wygasłego access_token
- `query_invoice_metadata.rs` - pobieranie listy faktur ze stronicowaniem
- `invoice_export.rs` - eksport faktur używany do przyrostowego pobiernia faktur
- `qrcode.rs` - generowanie kodu QR

## 🔧 Instalacja

Dodaj do `Cargo.toml`:

```toml
[dependencies]
ksef-client = "0.3"
tokio = { version = "1.37", features = ["full"] }
chrono = { version = "0.4", features = ["serde"]} 
```
