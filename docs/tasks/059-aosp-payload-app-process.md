# Task 059: AOSP Payload - Provide Real app_process and Runtime Libraries

Status: pending
Priority: critical
Type: feature
Parent: 031-aosp-payload-strategy

## Description
O payload atual em `payload/` contém apenas busybox, um init compilado e um `libc.so` vazio (0 bytes). Não há `app_process` (necessário para o zygote-light). Sem isso, nenhum app Android pode ser executado.

## Estado Atual do Payload
```
payload/
├── init.c                    # Fonte C do init
├── init_src.rs               # Fonte Rust do init
└── system/
    ├── bin/
    │   ├── busybox           # Funcional (1.1MB, estaticamente linkado)
    │   ├── init              # Compilado (783KB)
    │   ├── sh -> busybox     # Symlink funcional
    │   ├── cat/echo/ls/pwd/exit -> busybox
    │   └── (SEM app_process)  # ❌ FALTANDO
    └── lib64/
        └── libc.so           # ❌ VAZIO (0 bytes)
```

## O Que Falta (Mínimo para MVP)
1. **`system/bin/app_process`** ou **`system/bin/app_process64`** - Bootstrap do Android Runtime
2. **`system/lib64/libc.so`** - Bionic libc real (não placeholder vazio)
3. **`system/lib64/libandroid_runtime.so`** - Android runtime library
4. **`system/lib64/libart.so`** - ART (Android Runtime)
5. **`system/framework/`** - Framework JARs (boot.art, boot.oat)

## Estratégia Proposta
Conforme Task 031 (AOSP Payload Strategy), há 3 abordagens:

### A) Extrair de imagem AOSP (recomendado para MVP)
- Baixar system.img de um Generic System Image (GSI)
- Extrair binários x86_64 necessários
- Script para automatizar extração

### B) Compilar AOSP mínimo
- Build customizado com only `app_process` + deps
- Mais trabalhoso mas mais controlável

### C) Approach híbrido: init nativo + runtime AOSP
- Usar o init já existente (busybox-based)
- Apenas adicionar app_process e runtime libs do AOSP

## Todos
- [ ] Definir estratégia (A, B ou C)
- [ ] Criar script `scripts/fetch-payload.sh` para download/extração
- [ ] Obter `app_process64` para x86_64
- [ ] Obter bionic libc.so e linker64
- [ ] Obter ART runtime (libart.so, dalvikvm)
- [ ] Obter framework JARs mínimos
- [ ] Testar que `zygote::validate_runtime_layout()` passa
- [ ] Testar que `run-android-app run test.apk` avança além do erro atual
- [ ] Documentar origem e licenciamento dos binários AOSP

## Critério de Aceite
- `zygote::validate_runtime_layout()` retorna Ok
- `run-android-app run test.apk` tenta executar app_process (pode falhar por outros motivos)
- Payload tem tamanho razoável (< 500MB para MVP)
- Licenciamento documentado (Apache 2.0 do AOSP)
