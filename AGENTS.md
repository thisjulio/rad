# AGENTS.md

Este repositório implementa um **runner de apps Android no Linux** focado em **produtividade de desenvolvimento e debug**, com UX “um comando”:

> `run-android-app myapp.apk`

---

## Princípios do projeto

1. **Dev-first**: o principal usuário é quem quer iterar rápido (rodar, debugar, coletar logs, resetar ambiente).
2. **Sem VM**: sem emular hardware/rodar VM; o backend de execução usa Linux namespaces e recursos do kernel.
3. **Reprodutível**: todo app roda em um **prefix** isolado, com estado controlado.
4. **Pluggable backend**: a arquitetura deve permitir trocar componentes sem reescrever tudo.
5. **Compatibilidade honesta**: limites claros (ABIs, dependências de Play Services etc.) e mensagens de erro úteis.

---

## Artefatos e responsabilidades (papéis/“agentes”)

### 1) Agent: Product/UX (Dev Experience)
**Responsável por**: UX do CLI, mensagens de erro, fluxo “doctor”, integração com ferramentas de dev.
- Define comandos, flags, output padrão.
- Define layout de prefix e ergonomia (“reset”, “snapshot”, “bugreport”).
- Regras: todo erro deve apontar *o que faltou* e *como resolver*.

### 2) Agent: Runtime Orchestrator (Core)
**Responsável por**: ciclo de vida do “ambiente Android” por app/prefix.
- Criar prefix; extrair/montar payload AOSP; configurar runtime.
- Iniciar/parar processos e supervisionar.
- Políticas de caching e versionamento do payload.

### 3) Agent: Kernel/Sandbox (Linux)
**Responsável por**: isolamento e compatibilidade com kernel Linux.
- Namespaces (user/mount/pid/net), cgroups, seccomp, capabilities.
- binderfs (montagem e isolamento por instância quando aplicável).
- Diagnóstico do host (`doctor`): binderfs, permissões, cgroups, etc.
- Atenção a “rootless” vs “privileged”: design deve suportar ambos (mesmo que o MVP exija permissões).

### 4) Agent: APK Installer/Inspector
**Responsável por**: parse de APK e instalação no prefix.
- Leitura do Manifest, package name, version, ABIs (lib/x86_64 etc.).
- Estratégia de instalação e diretórios (/data/data/<pkg>).
- Geração de “compat report” por app (o que pode quebrar e por quê).

### 5) Agent: Debug/ADB Bridge
**Responsável por**: expor interfaces de debug como “device”.
- ADB over TCP/Unix socket; mapeamento por prefix.
- Comandos: `logcat`, `shell`, `bugreport`, `pull tombstones`.
- Integração com Android Studio (attach debugger quando possível).

---

## Git Worktrees para Trabalho Paralelo

Este repositório usa **Git Worktrees** para permitir que múltiplos agentes trabalhem simultaneamente em tasks diferentes.

### Estrutura de Worktrees

```
/home/thisjulio/Desktop/projects/rad/
├── rad.git/                    # Bare repository
├── main/                       # Worktree: main branch
├── task-021-pid-namespace/     # Worktree: task/021-pid-namespace
├── task-031-aosp-payload/      # Worktree: task/031-aosp-payload
└── experiments/                # Worktree: experiments branch
```

### Escolhendo o Worktree Correto

**Início de cada sessão:**
```bash
# 1. Verificar worktrees disponíveis
cd /home/thisjulio/Desktop/projects/rad/rad.git
git worktree list

# 2. Navegar para o worktree apropriado:
# - main: review, merge, testes gerais
# - task-NNN-*: desenvolvimento de task específica
# - experiments: testes destrutivos
cd /home/thisjulio/Desktop/projects/rad/WORKTREE_NAME
```

**Benefícios:**
- ✅ Múltiplos agentes podem trabalhar simultaneamente
- ✅ Cada worktree tem `target/` e `prefixes/` isolados
- ✅ Sem conflitos de checkout entre sessões
- ✅ Sem recompilação ao mudar de task

**Documentação completa:** Ver `/home/thisjulio/Desktop/projects/rad/WORKTREES.md`

---

## Protocolo de Orquestração de Tarefas (Git-Based + GitFlow)

Para garantir que múltiplos agentes (ou você e eu) saibam o progresso sem ferramentas externas, **seguimos GitFlow estritamente**:

### Fluxo de Trabalho (GitFlow)

1. **Saber "Onde Estamos"**:
   - O agente **SEMPRE** executa `git log -n 5`, `git branch` e `git status` no início de cada sessão.
   - Consultar `docs/tasks/` para identificar a próxima tarefa `[ ]` pendente.
   - Verificar se há mudanças remotas: `git fetch origin`.

2. **Estado WIP (Work In Progress)**:
   - A branch atual define a tarefa ativa: `task/NNN-slug` (ex: `task/001-user-ns`).
   - Se a branch for `master` ou `main`, nenhuma tarefa técnica está sendo executada.
   - **Nunca** commitar direto em `main` - sempre usar task branches.

3. **Início de Tarefa**:
   - Comando: `git checkout main && git pull origin main` (garantir base atualizada).
   - Criar branch: `git checkout -b task/NNN-slug`.
   - O agente deve atualizar o arquivo correspondente em `docs/tasks/NNN-*.md` marcando o status como em andamento.

4. **Desenvolvimento (TDD Flow)**:
   - **Passo Red**: Adicione um teste em `src/lib.rs` ou `tests/` que descreva o comportamento esperado da tarefa.
   - **Passo Green**: Implemente a lógica necessária. Execute `cargo test` para validar.
   - **Passo Refactor**: Limpe o código, melhore nomes e documentação.
   - **Commit**: Seguir o padrão: `task(NNN): descrição curta do que foi feito`.
   - Commits incrementais são encorajados (não esperar tarefa completa para commitar).

5. **Finalização de Tarefa**:
   - O agente deve rodar os testes/check de qualidade: `cargo test && cargo clippy`.
   - Atualizar o arquivo da tarefa marcando as checkboxes `[x]`.
   - **Merge para main**: 
     ```bash
     git checkout main
     git pull origin main  # CRÍTICO: garantir que main está atualizado
     git merge task/NNN-slug --no-ff  # Preservar histórico da branch
     ```
   - Se houver conflitos: resolver, testar novamente, e commitar merge.

6. **Sincronização com Remoto (OBRIGATÓRIO)**:
   - Após merge bem-sucedido, **SEMPRE** fazer push:
     ```bash
     git push origin main
     git push origin task/NNN-slug  # Opcional: preservar task branch no remoto
     ```
   - **ATENÇÃO**: Antes de push, verificar se há atualizações remotas:
     ```bash
     git fetch origin
     git status  # Verificar "Your branch is ahead/behind/diverged"
     ```
   - Se divergiu (trabalho paralelo detectado):
     ```bash
     git pull --rebase origin main  # Preferir rebase para histórico linear
     # OU
     git pull origin main  # Merge se preferir preservar histórico de merge
     ```
     - Resolver conflitos se houver.
     - Testar novamente: `cargo test`.
     - Então fazer push: `git push origin main`.

7. **Casos Especiais**:
   - **Push rejeitado** (`! [rejected]`): Significa que alguém fez push antes de você.
     ```bash
     git pull --rebase origin main
     cargo test  # Garantir que rebase não quebrou nada
     git push origin main
     ```
   - **Conflitos durante pull/rebase**: 
     - Resolver conflitos manualmente.
     - `git add <arquivos-resolvidos>`.
     - `git rebase --continue` (se rebase) OU `git commit` (se merge).
     - Testar: `cargo test`.
     - Push: `git push origin main`.
   - **Emergência** (precisa reverter): 
     ```bash
     git revert <commit-hash>  # Criar commit de reversão (preferido)
     # OU
     git reset --hard origin/main  # Descartar trabalho local (cuidado!)
     ```

### Regras de Ouro (Git)

- ✅ **SEMPRE** `git pull origin main` antes de criar task branch.
- ✅ **SEMPRE** `git pull origin main` antes de merge final.
- ✅ **SEMPRE** `git push origin main` após merge bem-sucedido.
- ✅ **SEMPRE** verificar `git status` antes de push.
- ✅ **SEMPRE** testar (`cargo test`) após merge/rebase/pull.
- ❌ **NUNCA** commitar direto em `main` (usar task branches).
- ❌ **NUNCA** fazer `git push --force origin main` (extremamente perigoso).
- ❌ **NUNCA** deixar trabalho sem push (outras sessões não verão).

---

## Convenções de desenvolvimento

### Linguagem/Stack
- Linguagem principal: **Rust**
- Async: `tokio` (quando necessário)
- CLI: `clap`
- Logs: `tracing` + `tracing-subscriber`
- Errors: `thiserror`, `anyhow`
- FS/OS: `nix`/`rustix` (evitar wrappers incompletos)

### Código
- Preferir módulos pequenos, com fronteiras claras (core vs backends).
- Exigir testes para parsing de APK e para “doctor”.
- Evitar “magia”: todas as ações críticas devem ser registradas em log.
- **Regras TDD**: Sempre verificar se a funcionalidade pode ser testada isoladamente. Testes que exigem capacidades de kernel devem ser marcados com `#[ignore]`.

---

## Estrutura do repositório

/crates
  /cli      -> parsing de args, UX, output
  /core     -> prefix, orquestração, estado
  /apk      -> parse/inspect/install
  /sandbox  -> namespaces, mount, binderfs, seccomp
  /adb      -> bridge e ferramentas de debug
/payload    -> embed/extract/mount do AOSP runtime
/docs       -> tarefas e documentação de design
/assets     -> artefatos estáticos

---

## Definition of Done (por PR)

- Compila em Linux x86_64 (CI).
- `run-android-app doctor` não regrede.
- Logs úteis (`RUST_LOG=info`).
- Documentação atualizada.
- Sem regressões em parsing de APK.
