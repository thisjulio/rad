# Task 003: Check Cgroups v2 Support

## Descrição Detalhada
O Android e os containers Linux modernos dependem do **Cgroups v2** (Control Groups) para gerenciar recursos (CPU, Memória, I/O) e para garantir que o ciclo de vida dos processos seja rastreado corretamente. Sem o Cgroups v2, o isolamento de recursos fica comprometido.

## Fluxo TDD
- [x] **Red**: Escrever teste que assevera que o sistema possui Cgroups v2 habilitado, falhando se o arquivo `/sys/fs/cgroup/cgroup.controllers` não for detectado.
- [x] **Green**: Implementar a verificação de existência do arquivo e o parse dos controladores disponíveis.
- [x] **Refactor**: Integrar o resultado no relatório final do comando `doctor`.

## Detalhes de Implementação (Rust)
1.  Verificar se o arquivo `/sys/fs/cgroup/cgroup.controllers` existe.
2.  Se existir, o sistema está usando Cgroups v2 (Unified Hierarchy).
3.  O "doctor" deve listar quais controladores estão habilitados (cpu, memory, io, pids).

## Referências
- [Control Group v2 Manual](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html)
