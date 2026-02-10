# Task 023: UID/GID Mapping (Rootless Container)

## Descrição Detalhada
Para rodar como usuário comum, precisamos mapear o UID do usuário host (ex: 1000) para o UID do root dentro do container (UID 0). Isso é feito escrevendo nos arquivos `/proc/self/uid_map` e `/proc/self/gid_map`.

## Fluxo TDD
- [x] **Red**: Teste que tenta ler o `uid_map` do processo sandbox e falha se não houver o mapeamento esperado.
- [x] **Green**: Implementar a escrita correta nos arquivos de mapeamento após o `CLONE_NEWUSER`.
- [x] **Refactor**: Garantir que o mapeamento suporte ranges maiores se necessário no futuro.

## Detalhes de Implementação (Rust)
1.  Obter o UID real usando `rustix::process::getuid`.
2.  Escrever a string `0 1000 1` (mapear UID 0 interno para UID 1000 externo, range de 1) no arquivo de mapeamento.
3.  Isso deve ser feito **após** o `CLONE_NEWUSER` mas **antes** de operações que exijam privilégios dentro do namespace.

## Referências
- [User Namespaces: uid_map documentation](https://man7.org/linux/man-pages/man7/user_namespaces.7.html)
