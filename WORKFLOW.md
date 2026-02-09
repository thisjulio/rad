# WORKFLOW.md

Este documento define o processo de desenvolvimento e rastreio de tarefas do `run-android-app`.

## Como saber o que está acontecendo (WIP)

Para identificar a tarefa atual, siga esta ordem de verificação:

1. **Branch Ativa**: 
   - Execute `git branch --show-current`.
   - Se o nome seguir o padrão `task/NNN-slug`, esta é a tarefa **Work In Progress**.
   
2. **Logs Recentes**:
   - Execute `git log -n 5 --oneline`.
   - Commits com o prefixo `task(NNN):` indicam o progresso atual dentro da tarefa.

3. **Status das Tarefas**:
   - Verifique os arquivos em `docs/tasks/`.
   - Tarefas com checkboxes `[ ]` vazios são candidatas a serem a **Próxima Tarefa**.
   - Tarefas com `[x]` estão concluídas e integradas na `master`.

## Ciclo de Vida de uma Tarefa

### 1. Preparação
- Escolha a próxima tarefa em `docs/tasks/` (ex: `001-check-user-namespaces.md`).
- Crie a branch: `git checkout -b task/001-user-ns`.

### 2. Desenvolvimento
- Realize commits atômicos: `git commit -m "task(001): implement proc check logic"`.
- Mantenha a branch focada **apenas** no escopo da tarefa.

### 3. Verificação
- Execute `cargo check` e `cargo test` (se aplicável).
- Rode o `run-android-app doctor` (se estiver trabalhando em diagnóstico).

### 4. Integração
- Marque as checkboxes no arquivo `.md` da tarefa.
- Volte para a master e integre:
  ```bash
  git checkout master
  git merge task/001-user-ns
  git branch -d task/001-user-ns
  ```

## Regras de Ouro
- Nunca trabalhe em duas tarefas simultaneamente na mesma branch.
- Se uma tarefa "explodir" em sub-tarefas novas, crie novos arquivos em `docs/tasks/` imediatamente.
- O histórico do Git é a **fonte da verdade** para a orquestração.
