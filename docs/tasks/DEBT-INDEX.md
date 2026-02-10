# Technical Debt Tasks - Index

Este arquivo rastreia débitos técnicos identificados em tasks marcadas como "completas" mas que contêm placeholders, stubs ou falta de testes.

## Débitos Ativos (6 tasks)

### 🔴 Critical Priority
- **[Task 053](053-debt-sandbox-implementation.md)**: Implementar orquestração real de namespaces
  - Parent: Task 003
  - Blocker para MVP
  - Stubs em `crates/sandbox/src/lib.rs` não fazem nada

### 🟠 High Priority
- **[Task 050](050-debt-apk-tests.md)**: Adicionar testes unitários para APK Inspector
  - Parent: Task 002
  - Violação do processo TDD
  - Zero testes para código funcional

- **[Task 051](051-debt-axml-parser.md)**: Implementar parser real de AndroidManifest.xml
  - Parent: Task 002
  - Package name é hardcoded como "com.example.placeholder"
  - Impede isolamento correto entre apps

### 🟡 Medium Priority
- **[Task 052](052-debt-compat-report.md)**: Implementar struct CompatReport
  - Parent: Task 002
  - Struct marcada como completa mas não existe
  - Sem análise de compatibilidade

- **[Task 054](054-debt-doctor-tests.md)**: Adicionar testes para doctor checks
  - Parent: Task 001
  - Código funcional mas sem testes
  - Não valida mensagens de erro

- **[Task 055](055-debt-prefix-tests.md)**: Adicionar testes para Prefix creation
  - Parent: Task 003
  - Código funcional mas sem testes
  - Não valida idempotência

## Estatísticas

- **Total de débitos**: 6
- **Críticos**: 1
- **Alta prioridade**: 2
- **Média prioridade**: 3
- **Débitos bloqueando MVP**: 2 (053, 051)

## Processo de Quitação

1. Cada task de débito deve seguir o fluxo TDD (Red/Green/Refactor)
2. Ao completar um débito, atualizar a task parent para refletir o estado real
3. Débitos críticos devem ser resolvidos antes de novas features
4. Pull requests devem incluir resolução de pelo menos 1 débito quando possível

## Relação com Tasks Originais

| Task Original | Status Oficial | Status Real | Débitos Criados |
|--------------|----------------|-------------|-----------------|
| 001-setup-workspace | completed | 90% | 054 |
| 002-apk-inspector | completed | 60% | 050, 051, 052 |
| 003-runtime-orchestrator | in_progress | 40% | 053, 055 |

## Próximos Débitos a Identificar

Quando mais tasks forem marcadas como "completas", este índice deve ser atualizado com novos débitos encontrados.
