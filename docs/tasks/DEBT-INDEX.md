# Technical Debt Tasks - Index

Este arquivo rastreia débitos técnicos identificados em tasks marcadas como "completas" mas que contêm placeholders, stubs ou falta de testes.

**Última atualização**: 2026-02-10 (auditoria funcional completa)

## Débitos Ativos (10 tasks)

### 🔴 Critical Priority

- **[Task 059](059-aosp-payload-app-process.md)**: Payload AOSP não tem app_process
  - Sem `app_process`, nenhum app Android pode ser executado
  - `libc.so` no payload é arquivo vazio (0 bytes)
  - **Blocker absoluto para MVP**

- **[Task 057](057-debt-shell-chroot-failure.md)**: Comando shell falha após chroot (ENOENT)
  - Parent: Task 006
  - Shell resolve path do HOST mas executa dentro do chroot
  - Impede uso interativo do sandbox

### 🟠 High Priority

- **[Task 060](060-integrate-runtime-crate.md)**: Integrar crate runtime no sandbox
  - Parent: Task 033
  - ServiceRegistry existe mas é isolado (nenhum crate usa)
  - Sem transporte Binder real

- **[Task 061](061-debt-wayland-protocol-integration.md)**: Implementar protocolo Wayland real
  - Parent: Tasks 042, 043
  - Tasks 042/043 marcadas como `[x]` mas NÃO usam nenhuma API Wayland
  - `wayland-client` e `wayland-protocols` declarados mas nunca importados
  - `FrameProvider` trait sem implementações

- **[Task 050](050-debt-apk-tests.md)**: Adicionar testes unitários para APK Inspector
  - Parent: Task 002
  - **Parcialmente resolvido**: 3 testes funcionais existem agora
  - Faltam: testes para erros, ZIP inválido, fixture APK (sem depender de `real.apk`)

### 🟡 Medium Priority

- **[Task 052](052-debt-compat-report.md)**: Implementar struct CompatReport
  - Parent: Task 002
  - Struct marcada como completa mas não existe
  - Sem análise de compatibilidade

- **[Task 056](056-debt-dead-code-cleanup.md)**: Limpeza de código morto e deps não utilizadas
  - Arquivo morto: `sandbox/src/wayland.rs` (nunca compilado)
  - Deps mortas: `quick-xml`, `sandbox`/`adb` no CLI
  - Structs duplicadas: `VirtualBuffer` vs `DmabufBuffer`

- **[Task 058](058-debt-doctor-weak-namespace-check.md)**: Doctor usa check fraco de namespaces
  - Parent: Task 001
  - Verifica apenas existência de path vs fork+unshare real

- **[Task 063](063-implement-stop-command.md)**: Implementar comando stop
  - Parent: Task 005 (4/5 completo)
  - Único subcomando pendente

### 🟢 Low Priority

- **[Task 062](062-implement-logs-follow.md)**: Implementar --follow no logs
  - Flag aceito pelo parser mas ignorado na implementação

## Débitos RESOLVIDOS (removidos desta lista)

| Débito | Data Resolução | Notas |
|--------|---------------|-------|
| Task 053 (Sandbox implementation) | 2026-02-09 | enter_namespaces(), setup_mounts(), chroot(), exec() implementados |
| Task 051 (AXML parser) | 2026-02-09 | axmldecoder integrado, package name extraído corretamente |
| Task 054 (Doctor tests) | 2026-02-09 | 3 testes em core::doctor + 4 em sandbox::doctor |
| Task 055 (Prefix tests) | 2026-02-09 | 5 testes significativos em core::prefix |

## Estatísticas

- **Total de débitos ativos**: 10
- **Críticos (MVP blockers)**: 2 (059, 057)
- **Alta prioridade**: 3 (060, 061, 050)
- **Média prioridade**: 4 (052, 056, 058, 063)
- **Baixa prioridade**: 1 (062)
- **Resolvidos**: 4 (053, 051, 054, 055)

## Discrepâncias Tasks vs Código (Auditoria 2026-02-10)

### Tasks marcadas `[x]` que NÃO estão realmente completas:
| Task | Claim | Realidade |
|------|-------|-----------|
| 042 (SurfaceFlinger) | Interceptação via Gralloc/VirGL | Apenas VirtualBuffer + FrameProvider trait sem impl |
| 043 (DMA-BUF) | Extensão linux-dmabuf + commit | DmabufBuffer + manager, mas ZERO chamadas Wayland |
| 053 checkbox | setup_mounts monta /proc, /sys | /proc e /sys são SKIPPED (warn + TODO task/021) |

### Tasks marcadas como pending que já foram resolvidas:
| Task | Status no arquivo | Realidade |
|------|-------------------|-----------|
| 051 (AXML parser) | pending | RESOLVIDO - axmldecoder funcional |
| 054 (Doctor tests) | pending | PARCIALMENTE RESOLVIDO - 7 testes existem |
| 055 (Prefix tests) | pending | PARCIALMENTE RESOLVIDO - 5 testes existem |

## Processo de Quitação

1. Cada task de débito deve seguir o fluxo TDD (Red/Green/Refactor)
2. Ao completar um débito, atualizar a task parent para refletir o estado real
3. Débitos críticos devem ser resolvidos antes de novas features
4. Pull requests devem incluir resolução de pelo menos 1 débito quando possível
