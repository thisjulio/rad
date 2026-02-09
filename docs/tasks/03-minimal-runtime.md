# Task 03: Minimal Android Runtime (AOSP-min)

**Objetivo**: Subir os serviços básicos necessários para carregar uma Activity.

## Sub-tarefas
- [ ] **Payload Extraction**:
    - Extrair o runtime AOSP mínimo (ART, Bionic, Libs básicas).
- [ ] **Zygote-light**:
    - Implementar um launcher que inicializa a ART (Android Runtime).
    - Carregar classes base do framework.
- [ ] **SystemServer Mock/Minimal**:
    - Iniciar `ActivityManager` e `PackageManager` mínimos.
    - Responder a intents de launch básicos.

## Referências
- [Android Boot Process (Zygote/SystemServer)](https://source.android.com/docs/core/architecture/boot-kernel)
- [ART Runtime documentation](https://source.android.com/docs/core/runtime)
