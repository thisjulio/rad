# Task 01: APK Inspector & Installer

**Objetivo**: Analisar o APK para garantir compatibilidade e preparar o diretório de instalação no Prefix.

## Sub-tarefas
- [ ] **ABI Detection**:
    - Abrir APK (zip).
    - Listar conteúdo de `lib/`.
    - Validar se existe `x86_64` ou se o app é puramente DEX (sem pastas de arquitetura).
- [ ] **Manifest Parsing**:
    - Extrair `AndroidManifest.xml` (binário).
    - Usar um parser AXML para obter `packageName`, `versionCode` e `mainActivity`.
- [ ] **Prefix Setup**:
    - Criar estrutura de diretórios em `~/.local/share/run-android-app/prefixes/<pkg>`.
    - Mapear `/data/data/<pkg>`, `/sdcard`, etc.

## Referências
- [Android APK Format](https://developer.android.com/studio/build/apk)
- [Rust `zip` crate](https://docs.rs/zip/latest/zip/)
- [AXML format specification](https://github.com/google/android-arscblaster)
