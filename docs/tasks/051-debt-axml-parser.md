# Task 051: [DEBT] Implement Real AndroidManifest.xml Parser

Status: pending
Priority: high
Type: technical-debt
Parent: 002-apk-inspector

## Description
A Task 002 marcou "Extract package name from AndroidManifest.xml (placeholder)" como completo, mas a implementação atual retorna um valor hardcoded. Para o MVP funcionar, precisamos extrair o package name real do APK.

## Débito Identificado
Em `crates/apk/src/lib.rs:75`:
```rust
// Placeholder for package name extraction (AXML parsing)
// In a real scenario, we would parse AndroidManifest.xml
let package_name = "com.example.placeholder".to_string();
```

**Problema**: Todo APK retorna o mesmo package name, impedindo múltiplos apps de terem prefixes isolados.

## Todos
- [ ] **Red**: Criar teste que valida package name de um APK real
- [ ] **Green**: Adicionar dependência `axml-parser` ou similar ao `crates/apk/Cargo.toml`
- [ ] Implementar função `parse_manifest(archive: &ZipArchive) -> Result<String>`
- [ ] Extrair arquivo `AndroidManifest.xml` do ZIP
- [ ] Parse do formato binário AXML
- [ ] Extrair atributo `package` do elemento `<manifest>`
- [ ] **Refactor**: Tratar erros (manifest ausente, AXML corrompido)
- [ ] Adicionar testes para diferentes formatos de manifest

## Fluxo TDD
1. **Red**: Teste que extrai package de um APK real deve falhar com "com.example.placeholder"
2. **Green**: Implementar parser AXML que retorna package name correto
3. **Refactor**: Abstrair parsing em módulo `crates/apk/src/manifest.rs`

## Referências
- Crate sugerido: [`abxml`](https://crates.io/crates/abxml) (Android Binary XML parser)
- Especificação AXML: https://github.com/google/android-arsclib
- Task relacionada: `012-axml-parsing-manifest.md`

## Critério de Aceite
- APKs diferentes devem retornar package names diferentes
- Prefixes devem ser criados com nome correto: `prefixes/com.real.app/`
- Testes devem validar pelo menos 3 APKs reais
