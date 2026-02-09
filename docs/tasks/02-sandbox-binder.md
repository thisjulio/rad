# Task 02: Sandbox & Binder Orchestration

**Objetivo**: Criar o isolamento de processo e fornecer um barramento Binder privado para o App.

## Sub-tarefas
- [ ] **Container Launch**:
    - Usar `unshare` (ou equivalent syscalls em Rust) para criar namespaces: `Mount`, `UTS`, `IPC`, `Net`, `PID`.
    - Pivot root ou Chroot para o ambiente AOSP mínimo.
- [ ] **Binderfs Setup**:
    - Montar uma instância privada do `binderfs` dentro do Mount Namespace do app.
    - Criar device nodes específicos (`binder`, `hwbinder`, `vndbinder`) via ioctl no `binder-control`.
- [ ] **Permissions Management**:
    - Configurar mapeamento de UID/GID (rootless se possível).

## Referências
- [rsbinder-tools (Reference implementation)](https://github.com/neofelis/rsbinder)
- [Mounting binderfs in namespaces](https://brauner.io/2019/01/09/android-binderfs.html)
