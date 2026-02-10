# Git Worktrees Setup - rad (run-android-app)

Este repositório usa **Git Worktrees** para permitir que **múltiplos agentes trabalhem em tasks diferentes simultaneamente**.

## 📁 Estrutura de Diretórios

```
/home/thisjulio/Desktop/projects/rad/
├── rad.git/                    # Bare repository (compartilhado)
│   ├── worktrees/              # Metadata dos worktrees
│   └── objects/                # Git objects (compartilhados)
│
├── main/                       # Worktree: main (produção)
│   ├── crates/
│   ├── payload/
│   ├── prefixes/               # Isolado por worktree
│   ├── target/                 # Isolado por worktree
│   └── .git -> rad.git
│
├── task-021-pid-namespace/     # Worktree: task/021-pid-namespace
│   ├── crates/
│   ├── payload/
│   ├── prefixes/               # Isolado
│   ├── target/                 # Isolado
│   └── .git -> rad.git
│
├── task-031-aosp-payload/      # Worktree: task/031-aosp-payload
│   ├── crates/
│   ├── payload/
│   │   └── system/             # AOSP payload (pode ser GB)
│   └── .git -> rad.git
│
└── experiments/                # Worktree: experiments
    └── .git -> rad.git
```

## 🤖 Uso por Agentes (Sessões de IA)

### Agente 1: Trabalhando em PID Namespace
```bash
cd /home/thisjulio/Desktop/projects/rad/task-021-pid-namespace
git status  # branch: task/021-pid-namespace
cargo test
cargo run -- doctor
# Commits vão para task/021-pid-namespace
```

### Agente 2: Trabalhando em AOSP Payload (simultaneamente!)
```bash
cd /home/thisjulio/Desktop/projects/rad/task-031-aosp-payload
git status  # branch: task/031-aosp-payload
./scripts/extract_aosp.sh
cargo build
# Commits vão para task/031-aosp-payload
# ✅ Não interfere com Agente 1
```

### Agente 3: Review/Testes em Main
```bash
cd /home/thisjulio/Desktop/projects/rad/main
git status  # branch: main
cargo test --all
cargo clippy
```

## 🔧 Comandos Úteis

### Listar Worktrees
```bash
cd /home/thisjulio/Desktop/projects/rad/rad.git
git worktree list
```

### Criar Novo Worktree para Task
```bash
cd /home/thisjulio/Desktop/projects/rad/rad.git

# Para nova task branch
git worktree add ../task-NNN-description -b task/NNN-description

# Para branch existente
git worktree add ../task-NNN-description task/NNN-description
```

### Remover Worktree (após merge)
```bash
cd /home/thisjulio/Desktop/projects/rad/rad.git
git worktree remove ../task-021-pid-namespace
# Ou manualmente:
rm -rf ../task-021-pid-namespace
git worktree prune
```

### Sincronizar com Remoto (de qualquer worktree)
```bash
cd /home/thisjulio/Desktop/projects/rad/task-021-pid-namespace
git fetch origin
git pull origin main  # Para atualizar base antes de merge
git push origin task/021-pid-namespace
```

## ✅ Benefícios

### 1. Isolamento Total
- Cada worktree tem seu próprio `target/` (build isolado)
- Cada worktree tem seu próprio `prefixes/` (dados de runtime isolados)
- Mudanças em um worktree não afetam outros

### 2. Trabalho Paralelo
- Múltiplos agentes podem trabalhar simultaneamente
- Sem conflitos de checkout
- Sem recompilação ao mudar de task

### 3. Economia de Espaço
- `.git` compartilhado (objects, refs)
- Apenas código-fonte e builds são duplicados
- ~13 MB por worktree adicional (sem builds)

### 4. GitFlow Simplificado
```bash
# Agente trabalhando em task/021
cd /home/thisjulio/Desktop/projects/rad/task-021-pid-namespace
git checkout main
git pull origin main
git checkout task/021-pid-namespace
git merge main  # Atualiza com latest main
# ... desenvolvimento ...
git commit -m "task(021): implement PID namespace"

# Quando pronto para merge
cd /home/thisjulio/Desktop/projects/rad/main
git pull origin main
git merge task/021-pid-namespace
git push origin main
```

## ⚠️ Limitações

### Uma Branch por Vez
```bash
# ❌ Não pode ter mesma branch em 2 worktrees
git worktree add ../copy1 main  # OK
git worktree add ../copy2 main  # ERRO

# ✅ Use branches diferentes
git worktree add ../copy2 -b main-experiment main
```

### Branches Compartilhadas
- Commits em uma branch são visíveis em todos worktrees
- `git status` mostra apenas mudanças locais do worktree
- `git branch` mostra todas as branches (compartilhadas)

## 📊 Disk Space

```
rad.git/                    11 MB   (bare repo)
main/                       13 MB   (código sem build)
task-021-pid-namespace/     13 MB   (código sem build)
task-031-aosp-payload/      13 MB   (código sem build)
experiments/                13 MB   (código sem build)

Total sem builds:           ~63 MB

Com builds (cargo):
main/                       ~500 MB  (código + target/)
task-021-pid-namespace/     ~500 MB  (código + target/)
task-031-aosp-payload/      ~500 MB  (código + target/)

Total com builds:           ~1.5 GB
```

## 🚀 Workflow Recomendado

### Sessão Normal (Agente escolhe worktree baseado em task)
```bash
# Início de sessão - Agente sempre verifica onde está
cd /home/thisjulio/Desktop/projects/rad
git worktree list  # Ver worktrees disponíveis

# Escolher worktree baseado na task
cd /home/thisjulio/Desktop/projects/rad/task-021-pid-namespace

# Verificar estado
git status
git log -n 5

# Trabalhar normalmente
cargo test
git add .
git commit -m "task(021): implement feature"
git push origin task/021-pid-namespace
```

### Sessão de Review
```bash
cd /home/thisjulio/Desktop/projects/rad/main
git pull origin main
cargo test --all
cargo clippy
```

### Limpeza Periódica
```bash
# Remover worktrees de tasks já merged
cd /home/thisjulio/Desktop/projects/rad/rad.git
git worktree list
git worktree remove ../task-021-pid-namespace  # Se já foi merged

# Limpar branches remotas obsoletas
git fetch --prune origin
git branch -d task/021-pid-namespace  # Se já foi merged
```

## 🤖 Instruções para Agentes (IA)

### Início de Sessão
```bash
# 1. Verificar estrutura de worktrees
cd /home/thisjulio/Desktop/projects/rad/rad.git
git worktree list

# 2. Escolher worktree baseado na task a executar
# - main: para review, merge, testes gerais
# - task-NNN-*: para desenvolvimento de task específica
# - experiments: para testes destrutivos/experimentais

# 3. Navegar para o worktree escolhido
cd /home/thisjulio/Desktop/projects/rad/WORKTREE_NAME

# 4. Verificar estado Git
git status
git log -n 5
git fetch origin
```

### Durante Desenvolvimento
```bash
# Trabalhar normalmente no worktree
cargo build
cargo test
git add .
git commit -m "task(NNN): description"

# Sincronizar com remoto
git fetch origin
git pull origin main  # Atualizar base
git push origin BRANCH_NAME
```

### Ao Finalizar Task
```bash
# No worktree da task
cargo test && cargo clippy
git push origin BRANCH_NAME

# Mudar para main worktree
cd /home/thisjulio/Desktop/projects/rad/main
git pull origin main
git merge BRANCH_NAME
git push origin main

# Opcionalmente remover worktree
cd /home/thisjulio/Desktop/projects/rad/rad.git
git worktree remove ../task-NNN-description
```

## 📝 Notas

- **rad.git** é o repositório central (bare). Nunca trabalhe diretamente nele.
- Cada worktree é um diretório de trabalho normal com `.git` linkado ao bare repo.
- Você pode deletar qualquer worktree sem perder dados (commits estão no bare repo).
- O diretório `funnyidea-old/` (fora de rad/) é backup do setup antigo. Pode ser deletado após validação.

## 🔗 Referências

- [Git Worktree Documentation](https://git-scm.com/docs/git-worktree)
- `main/AGENTS.md` - Protocolo GitFlow completo
