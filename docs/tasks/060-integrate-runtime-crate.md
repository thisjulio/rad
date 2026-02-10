# Task 060: Integrate Runtime Crate into Sandbox Execution

Status: pending
Priority: high
Type: feature
Parent: 033-minimal-system-services

## Description
O crate `runtime` tem stubs funcionais para ActivityManager e PackageManager (com ServiceRegistry, handle_call, etc), mas **não é usado por nenhum outro crate**. Os service stubs existem em isolamento total.

## Estado Atual
- `crates/runtime/src/lib.rs`: ServiceRegistry com ActivityManager e PackageManager stubs
- **Nenhum outro crate depende de `runtime`**
- Não há transporte Binder real (handle_call aceita `&[u8]`, mas ninguém chama)
- `rsbinder` está no workspace `Cargo.toml` mas **não é usado** pelo runtime crate

## O Que Falta
1. **Exposição via Binder**: Os stubs precisam escutar em device nodes do binderfs
2. **Integração com sandbox**: O ServiceRegistry precisa ser iniciado dentro do namespace
3. **Transporte real**: Conectar `handle_call` ao protocolo Binder via `rsbinder`

## Abordagem Proposta
1. Adicionar `rsbinder` como dependência do crate `runtime`
2. Implementar listener Binder que roteia chamadas para ServiceRegistry
3. No `core::prefix::run_in_sandbox_child()`, chamar `runtime::init_minimal_services()` antes do exec
4. Montar binderfs device no sandbox e expor ServiceManager

## Todos
- [ ] Adicionar `runtime` como dependência de `core`
- [ ] Chamar `init_minimal_services()` no setup do sandbox
- [ ] Implementar transporte Binder básico com `rsbinder`
- [ ] Conectar ServiceRegistry ao Binder device node
- [ ] Testar que ActivityManager.checkPermission() responde dentro do sandbox
- [ ] Documentar arquitetura do service pipeline

## Critério de Aceite
- Processos no sandbox podem chamar `ServiceManager.getService("activity")`
- `checkPermission()` retorna PERMISSION_GRANTED
- `cargo test` passa com testes de integração
