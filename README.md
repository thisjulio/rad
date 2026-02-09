# README.md

# run-android-app

`run-android-app` é um **runner de apps Android no Linux**, voltado para **desenvolvedores** que querem **rodar e debugar** um APK sem configurar AVD/emulador manualmente.

Objetivo de UX:

```bash
run-android-app myapp.apk
```

O projeto mira Linux x86_64 e prioriza:

- Execução rápida
- Logs e diagnóstico
- Ambiente reproduzível por app (prefix)
- Integração de debug (ADB/logcat)

> **Nota**: “sem emulador” aqui significa sem VM/AVD. Ainda assim, um APK depende de runtime/serviços Android; o runner fornece isso via um backend de execução.

## Status
Projeto pessoal em desenvolvimento. Os documentos de produto e agentes estão em:
- [PRODUCT.md](./PRODUCT.md)
- [AGENTS.md](./AGENTS.md)

## Instalação
### Pré-requisitos (host)
- Linux x86_64
- Kernel com suporte a namespaces (user/mount/pid) e permissões para utilizá-los
- `binderfs` (para IPC Android), quando exigido pelo backend
- Ferramentas básicas de build (Rust)

### Build
```bash
cargo build --release
./target/release/run-android-app --help
```

## Uso
### Rodar um APK
```bash
run-android-app myapp.apk
```
### Ver logs
```bash
run-android-app logcat myapp.apk
```
### Abrir shell no ambiente
```bash
run-android-app shell myapp.apk
```
### Parar execução
```bash
run-android-app stop myapp.apk
```
### Resetar o ambiente (prefix)
```bash
run-android-app reset myapp.apk
```
### Diagnóstico do host
```bash
run-android-app doctor
```

## Conceitos
### Prefix (isolamento por app)
Cada app roda dentro de um “prefix” com estado próprio, similar ao conceito do Wine:
```text
~/.local/share/run-android-app/prefixes/<package-name>/
  data/            -> dados do app
  cache/           -> cache
  logs/            -> logcat, tombstones, anr
  overrides.toml   -> overrides/perfis
```

### Payload (runtime Android mínimo)
O runner precisa de um “payload” que inclui runtime/serviços mínimos (AOSP). O payload pode ser:
- Embutido no binário (single-binary) e extraído/montado sob demanda, ou
- Baixado/gerado na primeira execução (conforme estratégia do projeto)

## Limitações (intencionais no início)
- Foco em apps Java/Kotlin puros ou que incluam `lib/x86_64/` no APK.
- APKs que dependem de Google Play Services podem não funcionar no MVP.
- Recursos de hardware (câmera, sensores) podem ser limitados.
- UI “perfeita” (Wayland/X11, GPU) pode entrar depois do MVP (modo headless pode existir no começo).

## Troubleshooting
- **`doctor` aponta falta de binderfs / permissões**: Verifique se o kernel tem binderfs habilitado e se o usuário tem permissões necessárias. Alguns modos podem exigir capabilities ou execução privilegiada.
- **O APK tem libs ARM e não roda**: Se o APK contém apenas `lib/arm64-v8a/` e não contém `lib/x86_64/`, o runner não consegue executar “nativamente” em x86_64.
- **Logs insuficientes**: Rode com `RUST_LOG=info run-android-app myapp.apk`.

## Segurança
O runner busca operar em sandbox (namespaces, permissões mínimas) e manter isolamento por prefix. Ainda assim, rodar apps Android no desktop amplia superfície de ataque. Use com consciência, especialmente com APKs desconhecidos.

## Licença
A definir. Se o projeto distribuir partes do AOSP, manter compatibilidade com as licenças do AOSP e dependências.
