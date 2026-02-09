# Task 013: Prefix Filesystem Layout & Setup

## Descrição Detalhada
Assim como o Wine usa o "WINEPREFIX", o `run-android-app` deve criar um ambiente isolado por App. Isso garante que os dados de um App não interfiram em outro e permite o "reset" fácil.

## Fluxo TDD
- [ ] **Red**: Escrever teste que assevera que, após chamar a criação do prefix, os subdiretórios esperados (`root/`, `data/`, etc.) existem no sistema de arquivos.
- [ ] **Green**: Implementar `create_dir_all` e a lógica de verificação de existência.
- [ ] **Refactor**: Adicionar suporte a caminhos configuráveis e validação de permissões.

## Detalhes de Implementação (Rust)
1.  Criar a árvore de diretórios usando `std::fs::create_dir_all`.
2.  Garantir permissões corretas (0755).
3.  Implementar lógica de "idempotência" (não recriar se já existir, a menos que o flag `--reset` seja usado).
