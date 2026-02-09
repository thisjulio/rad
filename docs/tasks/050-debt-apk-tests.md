# Task 050: [DEBT] Add Unit Tests for APK Inspector

Status: pending
Priority: high
Type: technical-debt
Parent: 002-apk-inspector

## Description
A Task 002 foi marcada como completa mas não possui testes unitários reais. Segundo AGENTS.md, o projeto deve seguir TDD (Red/Green/Refactor), mas nenhum teste foi criado para o APK inspector.

## Débito Identificado
Em `crates/apk/src/lib.rs`:
- ✅ Código funcional existe (ApkInspector, Abi enum, ApkInfo struct)
- ❌ Nenhum teste unitário implementado
- ❌ Não há validação automatizada de detecção de ABIs
- ❌ Não há teste para erros (arquivo não existe, ZIP corrompido, etc)

## Todos
- [ ] **Red**: Criar teste `test_apk_abi_detection()` que valida ABIs detectadas
- [ ] **Green**: Criar APK de fixture com estrutura `lib/arm64-v8a/` e `lib/x86_64/`
- [ ] **Refactor**: Adicionar testes para casos de erro (arquivo não existe, ZIP inválido)
- [ ] Adicionar teste `test_apk_without_native_libs()` para apps puramente Java/Kotlin
- [ ] Adicionar teste `test_apk_all_abis()` com todas as 4 ABIs suportadas
- [ ] Adicionar teste `test_apk_invalid_zip()` que valida erro adequado

## Critério de Aceite
- `cargo test --package apk` deve executar pelo menos 4 testes
- Cobertura de código > 80% em `crates/apk/src/lib.rs`
- Todos os testes devem passar sem ignorar casos (#[ignore])

## Context
- `AGENTS.md` - Convenções TDD
- Task 002 original marcou "Add tests for APK inspection" como completo incorretamente
