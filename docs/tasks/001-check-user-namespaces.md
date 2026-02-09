# Task 001: Check User Namespaces Support

## Descrição Detalhada
Esta tarefa consiste em implementar a verificação de suporte a **User Namespaces** (user_ns) no sistema host. User Namespaces são fundamentais para permitir que o `run-android-app` execute em modo "rootless", garantindo que um processo possa ter privilégios de root dentro do container enquanto permanece um usuário comum no host.

A implementação em Rust deve verificar dois pontos principais:
1.  **Disponibilidade do Kernel**: Verificar se o kernel foi compilado com `CONFIG_USER_NS=y`. Isso pode ser inferido tentando criar um namespace ou checando arquivos em `/proc`.
2.  **Permissão de Clone Não Privilegiado**: Em distribuições como Debian e algumas versões do Ubuntu, a criação de user namespaces por usuários comuns é bloqueada por padrão. O valor de `/proc/sys/kernel/unprivileged_userns_clone` deve ser `1`.

## Detalhes de Implementação (Rust)
- Ler o arquivo `/proc/sys/kernel/unprivileged_userns_clone` usando `std::fs::read_to_string`.
- Se o arquivo não existir, o sistema pode ser antigo ou usar uma lógica diferente (como o Fedora, que não possui esse arquivo e permite por padrão).
- Tentar realizar uma chamada experimental de `unshare(CloneFlags::CLONE_NEWUSER)` usando o crate `nix` para validar na prática.

## Por que é importante?
Sem User Namespaces, precisaríamos de `sudo` para cada execução de app, o que quebra o princípio de UX "one command" e segurança do projeto.

## Referências
- [Baeldung: Linux kernel unprivileged_userns_clone](https://www.baeldung.com/linux/kernel-unprivileged-userns-clone)
- [Namespaces(7) Manual Page](https://man7.org/linux/man-pages/man7/namespaces.7.html)
