# Task 021: Namespace Orchestration (Unshare)

## Descrição Detalhada
Para isolar o processo Android do sistema host, utilizaremos os Namespaces do Linux. Esta é a base da nossa estratégia "sem VM".

## Fluxo TDD
- [ ] **Red**: Criar um teste de integração que tenta verificar se o PID atual mudou ou se o hostname foi isolado após a chamada da função de sandbox.
- [ ] **Green**: Implementar a chamada de `unshare` com as flags de namespace apropriadas.
- [ ] **Refactor**: Isolar a lógica de montagem do `/proc` para ser executada apenas dentro do novo namespace.

## Detalhes de Implementação (Rust)
1.  Usar `nix::sched::unshare` ou `rustix::process::unshare`.
2.  As flags necessárias são `CLONE_NEWNS | CLONE_NEWPID | CLONE_NEWUTS`.
3.  Após o `unshare`, o processo deve realizar o `mount` do `/proc` privado para que ferramentas como `top` ou `ps` funcionem corretamente dentro do container.

## Referências
- [Rust `nix` crate: sched](https://docs.rs/nix/latest/nix/sched/index.html)
- [Linux manual: unshare(2)](https://man7.org/linux/man-pages/man2/unshare.2.html)
