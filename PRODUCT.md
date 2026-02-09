# PRODUCT.md

# run-android-app — Produto

## Visão

Um **runner “um comando”** para executar e debugar apps Android no Linux, reduzindo o atrito de:
- configurar emulador/AVD
- lidar com imagens e setups manuais
- reproduzir bugs com ambientes inconsistentes

O norte do produto é: **ambiente determinístico por app** + **diagnóstico e debug fáceis**.

---

## Problema

Desenvolvedores Android (e times que dependem de apps Android) frequentemente precisam:
- validar correções rapidamente
- reproduzir bugs específicos
- coletar logs e crash dumps
- iterar sem depender de um device físico

AVDs/emuladores são pesados, exigem setup e nem sempre são “plug and play”. Dispositivo físico resolve, mas adiciona fricção e variabilidade (estado, permissões, versões).

---

## Público-alvo

### Persona 1: Dev Android
- Quer rodar o app rapidamente após `gradle assembleDebug`.
- Quer `logcat`, `tombstones`, e reset rápido do ambiente.

### Persona 2: QA/Engenharia de qualidade
- Quer reproduzir bugs em ambiente “limpo”.
- Quer snapshots e relatórios (bugreport) automatizados.

### Persona 3: CI/Automação
- Quer executar smoke tests sem VM pesada.
- Quer ambientes reprodutíveis e provisionamento previsível.

---

## Proposta de valor

- **Um binário** (`run-android-app`) e **um comando** para rodar.
- Prefix por app: **reset/snapshot** simples.
- `doctor`: valida host e explica correções.
- Interfaces de debug: logcat/shell/bugreport/ADB.
- Compatibilidade incremental via perfis e overrides (estilo Proton).

---

## O que NÃO é (no início)

- Não é “rodar qualquer APK do mundo”.
- Não é “substituir totalmente Android Studio/AVD”.
- Não promete Play Services no MVP.
- Não promete fidelidade total de hardware/sensores/câmera no começo.

---

## Requisitos do produto

### Funcionais (MVP)
1. `doctor` identifica pré-requisitos do host (kernel features/permissões) e aponta soluções.
2. `run-android-app <apk>`:
   - inspeciona APK (package, ABIs, dependências óbvias)
   - cria prefix
   - instala
   - executa
3. `logcat`, `shell`, `stop`, `reset`.
4. Coleta automática:
   - logs
   - crash dumps/tombstones
   - eventos ANR (quando aplicável)
5. Modo “compat report”:
   - diz se o APK parece Java-only
   - se contém `lib/x86_64`
   - se contém apenas ARM

### Não-funcionais
- Reprodutibilidade por prefix
- Logs úteis e diagnósticos claros
- Segurança razoável por sandbox
- “Single-binary distribution” (mesmo que extraia payload em runtime)

---

## Diferenciação (por que não “só usar Waydroid”)

Mesmo se o backend inicial usar conceitos similares (Android userspace em container), o produto difere por:
- Dev-first UX (doctor/logcat/shell/bugreport integrados)
- Prefix por app + reset/snapshot
- Overrides por app (base de compat)
- Integração com CI (headless, captura, bundles)

---

## Arquitetura do produto (alto nível)

### Componentes
- CLI (UX, comandos)
- Prefix manager (estado por app)
- APK inspector/installer
- Sandbox (namespaces, binderfs, mount)
- Runtime backend (pluggable):
  - Backend A: AOSP mínimo (compatibilidade primeiro)
  - Backend B: híbrido (substitui partes por nativo)
  - Backend C: nativo (reimplementações, longo prazo)
- Debug bridge (ADB/logcat)
- (Futuro) UI bridge (Wayland/X11)

---

## Roadmap sugerido

### Fase 0 — Fundamentos
- Layout de prefix + `doctor`
- Inspector de APK (ABIs, manifest)
- Orquestração básica

### Fase 1 — MVP executável
- Instalar e executar “hello world” Android
- `logcat`, `stop`, `reset`
- Captura de tombstones/anr

### Fase 2 — Debug sério
- Expor ADB por prefix
- Bugreport bundle (zip com logs + estado relevante)
- Snapshots básicos

### Fase 3 — UX estilo Proton
- Perfis/overrides por app
- Database de compat
- Fixups automáticos

### Fase 4 — UI/Integração
- Janela nativa no Wayland/X11
- Input/clipboard/dpi
- GPU e performance

### Fase 5 — Pesquisa “Wine-like” (anos)
- Substituir serviços do Android por equivalentes nativos incrementalmente
- Medir compatibilidade por classes de apps

---

## Riscos e mitigação

1. **ABIs e libs nativas**
   - Mitigação: escopo inicial x86_64 + Java-only.
2. **Kernel/permissões variam por distro**
   - Mitigação: `doctor` forte + modos privilegiados opcionais.
3. **Dependência de Play Services**
   - Mitigação: perfis; suporte opcional a alternativas; documentação honesta.
4. **Tamanho do payload**
   - Mitigação: cache e extração/mount incremental; estratégia de atualização.

---

## Métricas de sucesso (dev-first)

- Tempo do comando até app abrir (p50/p95)
- Tempo para obter logs de crash
- Taxa de reprodução de bug em prefix limpo
- Facilidade de setup (doctor resolve em minutos)
- Estabilidade (start/stop/reset sem “ficar sujo”)
