# Task 00: Host Diagnostic (The "Doctor")

**Objetivo**: Garantizar que o sistema host possui as capacidades de kernel necessárias para rodar o Android sem VM.

## Sub-tarefas
- [ ] Implementar check de **User Namespaces** (`/proc/sys/kernel/unprivileged_userns_clone` ou via syscall).
- [ ] Implementar check de **Binderfs**:
    - Verificar se `CONFIG_ANDROID_BINDERFS=y`.
    - Tentar encontrar o mount point atual de `binderfs`.
    - Verificar permissões de `/dev/binderfs/binder-control`.
- [ ] Implementar check de **Cgroups v2**: Necessário para controle de recursos e isolamento.
- [ ] Implementar check de **KVM** (opcional para v0, mas útil para performance se usarmos algum shim).

## Referências
- [Binderfs Kernel Doc](https://www.kernel.org/doc/html/latest/admin-guide/binderfs.html)
- [Namespaces(7) - Linux Manual Page](https://man7.org/linux/man-pages/man7/namespaces.7.html)
- [Rust `nix` crate for syscalls](https://docs.rs/nix/latest/nix/)
