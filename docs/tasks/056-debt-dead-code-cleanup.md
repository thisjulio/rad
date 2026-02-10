# Task 056: [DEBT] Dead Code & Unused Dependencies Cleanup

Status: pending
Priority: medium
Type: technical-debt
Parent: code-review-2026-02-10

## Description
Auditoria de 2026-02-10 encontrou código morto, dependências não utilizadas e arquivos órfãos no workspace.

## Problemas Identificados

### 1. Arquivo morto: `crates/sandbox/src/wayland.rs`
- Arquivo existe mas **não é declarado em `lib.rs`** (que só exporta `doctor` e `binderfs`)
- Nunca é compilado
- Usa imports que não existem no `wayland-client` 0.29 (`GlobalList`, `Display` direto, `xdg_wm_base` em wayland-client)
- Dependências Wayland não estão nem no `Cargo.toml` do sandbox

### 2. Dependência morta: `quick-xml` em `crates/apk/Cargo.toml`
- Declarada na linha 15: `quick-xml = "0.31"`
- **Nunca importada** em `src/lib.rs` — todo parsing AXML usa `axmldecoder`

### 3. Dependências mortas no CLI: `sandbox` e `adb`
- `crates/cli/Cargo.toml` declara `sandbox` e `adb` como dependências
- **Nenhum dos dois é importado** em `src/main.rs`
- CLI usa sandbox indiretamente via `core::prefix`

### 4. Dependências mortas em `crates/adb/Cargo.toml`
- Declara `tokio`, `tracing`, `anyhow`, `thiserror`
- Nenhuma é importada (crate contém apenas o template `add()`)

### 5. Structs duplicadas: `VirtualBuffer` vs `DmabufBuffer`
- `crates/wayland/src/lib.rs`: `VirtualBuffer` (OwnedFd + width/height/stride/format)
- `crates/wayland/src/dmabuf.rs`: `DmabufBuffer` (mesma coisa + offset + validação)
- Quase idênticos, devem ser unificados

### 6. Parâmetro ignorado: `build_ld_library_path`
- `crates/core/src/zygote.rs:53`: aceita `_prefix_root: &Path` mas ignora completamente

## Todos
- [ ] Remover `crates/sandbox/src/wayland.rs` (arquivo morto)
- [ ] Remover `quick-xml` do `crates/apk/Cargo.toml`
- [ ] Remover `sandbox` e `adb` do `crates/cli/Cargo.toml`
- [ ] Limpar dependências não utilizadas do `crates/adb/Cargo.toml`
- [ ] Unificar `VirtualBuffer` e `DmabufBuffer` ou documentar a distinção
- [ ] Usar o parâmetro `prefix_root` em `build_ld_library_path` ou removê-lo

## Critério de Aceite
- `cargo build` continua passando
- `cargo test` continua passando
- `cargo clippy` não reporta unused imports/deps
