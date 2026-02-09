# Task 011: APK ABI Detection

## Descrição Detalhada
Um APK é um arquivo ZIP que pode conter bibliotecas nativas compiladas para diferentes arquiteturas (ABIs). Para o MVP em Linux x86_64, precisamos de um módulo que inspecione o APK e decida se ele pode ser executado.

## Fluxo TDD
- [ ] **Red**: Criar testes unitários em `crates/apk/src/lib.rs` usando pequenos arquivos ZIP de teste (em memória ou temporários) que simulam APKs com diferentes estruturas de `lib/` (vazio, arm64, x86_64).
- [ ] **Green**: Implementar a lógica de inspeção usando o crate `zip`.
- [ ] **Refactor**: Melhorar a API para retornar um enum `CompatStatus` bem definido.

## Detalhes de Implementação (Rust)
1.  Usar o crate `zip` para abrir o APK sem extraí-lo totalmente (streaming).
2.  Percorrer a estrutura de diretórios procurando pela pasta `lib/`.
3.  **Lógica de Decisão**:
    - Se `lib/` não existe -> Retornar `CompatStatus::Universal`.
    - Se `lib/x86_64/` existe -> Retornar `CompatStatus::NativeX86_64`.
    - Se `lib/` existe mas não tem `x86_64` (ex: apenas `arm64-v8a`) -> Retornar `CompatStatus::Incompatible` (no v0).

## Referências
- [Android Developers: ABI Management](https://developer.android.com/ndk/guides/abis)
- [Rust `zip` Crate Documentation](https://docs.rs/zip/latest/zip/)
