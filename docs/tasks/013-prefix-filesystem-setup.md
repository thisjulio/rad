# Task 013: Prefix Filesystem Layout & Setup

## Descrição Detalhada
Assim como o Wine usa o "WINEPREFIX", o `run-android-app` deve criar um ambiente isolado por App. Isso garante que os dados de um App não interfiram em outro e permite o "reset" fácil.

## Estrutura Alvo
```text
~/.local/share/run-android-app/prefixes/<pkg>/
  ├── root/        # Mount point para o AOSP runtime
  ├── data/        # Mapeado para /data no Android
  ├── cache/       # Mapeado para /cache
  ├── dev/         # Mapeado para /dev (incluindo binderfs)
  └── config.toml  # Configurações específicas do prefix
```

## Detalhes de Implementação (Rust)
1.  Criar a árvore de diretórios usando `std::fs::create_dir_all`.
2.  Garantir permissões corretas (0755).
3.  Implementar lógica de "idempotência" (não recriar se já existir, a menos que o flag `--reset` seja usado).
