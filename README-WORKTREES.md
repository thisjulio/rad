# rad - Run Android Apps on Desktop

Este diretório contém múltiplos **Git Worktrees** para desenvolvimento paralelo.

## 📁 Estrutura

```
rad/
├── rad.git/                    # Bare repository (não trabalhar aqui)
├── main/                       # Worktree: branch main (produção)
├── task-021-pid-namespace/     # Worktree: desenvolvimento de task 021
├── task-031-aosp-payload/      # Worktree: desenvolvimento de task 031
├── experiments/                # Worktree: testes experimentais
└── WORKTREES.md               # Documentação completa de worktrees
```

## 🚀 Início Rápido

### Para Agentes (IA)
```bash
# 1. Verificar worktrees disponíveis
cd /home/thisjulio/Desktop/projects/rad/rad.git
git worktree list

# 2. Escolher worktree baseado na task
cd /home/thisjulio/Desktop/projects/rad/main              # Para review/merge
cd /home/thisjulio/Desktop/projects/rad/task-021-pid-namespace  # Para task 021
cd /home/thisjulio/Desktop/projects/rad/task-031-aosp-payload   # Para task 031

# 3. Trabalhar normalmente
git status
cargo test
git commit -m "task(NNN): description"
git push origin BRANCH_NAME
```

### Para Desenvolvimento Manual
```bash
# Navegar para o worktree desejado
cd main/              # Branch main
cd task-021-*/        # Task 021
cd task-031-*/        # Task 031
cd experiments/       # Experimentos

# Desenvolver normalmente
cargo build
cargo test
git add .
git commit
```

## 📊 Espaço em Disco

- **rad.git**: 11 MB (bare repository compartilhado)
- **main**: 575 MB (com build completo)
- **task-021-pid-namespace**: 13 MB (sem build)
- **task-031-aosp-payload**: 13 MB (sem build)
- **experiments**: 13 MB (sem build)

**Total**: ~625 MB (com apenas main compilado)

## 📖 Documentação

- **WORKTREES.md** - Guia completo de git worktrees
- **main/AGENTS.md** - Protocolo GitFlow e instruções para agentes
- **main/README.md** - Documentação do projeto

## 🔗 Links

- GitHub: https://github.com/thisjulio/rad
- Bare Repo: `rad.git/`
- Main Worktree: `main/`
