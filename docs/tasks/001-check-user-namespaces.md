# Task 001: Check User Namespaces Support

## Descrição Detalhada
Esta tarefa consiste em implementar a verificação de suporte a **User Namespaces** (user_ns) no sistema host. User Namespaces são fundamentais para permitir que o `run-android-app` execute em modo "rootless", garantindo que um processo possa ter privilégios de root dentro do container enquanto permanece um usuário comum no host.

## Fluxo TDD
- [ ] **Red**: Criar um teste unitário em `crates/sandbox/src/lib.rs` que tenta validar a presença de user namespaces e falha (ex: asseverando um resultado de sucesso em um mock controlado ou simulando a leitura do arquivo `/proc`).
- [ ] **Green**: Implementar a lógica de leitura de `/proc/sys/kernel/unprivileged_userns_clone` e a tentativa de `unshare(CLONE_NEWUSER)`.
- [ ] **Refactor**: Refinar o tratamento de erros usando `thiserror` e garantir que a função retorne um `Result` claro para o CLI.

## Detalhes de Implementação (Rust)
- Ler o arquivo `/proc/sys/kernel/unprivileged_userns_clone` usando `std::fs::read_to_string`.
- Se o arquivo não existir, o sistema pode ser antigo ou usar uma lógica diferente (como o Fedora, que não possui esse arquivo e permite por padrão).
- Tentar realizar uma chamada experimental de `unshare(CloneFlags::CLONE_NEWUSER)` usando o crate `nix` para validar na prática.

## Referências
- [Baeldung: Linux kernel unprivileged_userns_clone](https://www.baeldung.com/linux/kernel-unprivileged-userns-clone)
- [Namespaces(7) Manual Page](https://man7.org/linux/man-pages/man7/namespaces.7.html)
