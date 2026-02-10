# Task 051: [DEBT] Implement Real AndroidManifest.xml Parser

Status: completed
Priority: high
Type: technical-debt
Parent: 002-apk-inspector

## Description
A Task 002 marcou "Extract package name from AndroidManifest.xml (placeholder)" como completo, mas a implementação original retornava um valor hardcoded.

## Resolução (2026-02-09)
O débito foi resolvido. A implementação atual usa `axmldecoder` para parsing real:
- `crates/apk/src/lib.rs:90-110`: `extract_package_name()` decodifica AXML binário e lê atributo `package`
- `crates/apk/src/lib.rs:138-188`: `parse_manifest()` extrai versionCode, versionName e mainActivity
- 3 testes funcionais validam extração correta (usando F-Droid APK)

## Todos (originais - todos resolvidos)
- [x] **Red**: Criar teste que valida package name de um APK real
- [x] **Green**: Adicionar dependência `axmldecoder` ao `crates/apk/Cargo.toml`
- [x] Implementar função `parse_manifest(archive: &ZipArchive) -> Result<AppManifest>`
- [x] Extrair arquivo `AndroidManifest.xml` do ZIP
- [x] Parse do formato binário AXML
- [x] Extrair atributo `package` do elemento `<manifest>`
- [x] **Refactor**: Tratar erros (manifest ausente, AXML corrompido)
- [ ] Adicionar testes para diferentes formatos de manifest (pendente - ver Task 050)

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
