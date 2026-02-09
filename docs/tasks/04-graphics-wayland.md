# Task 04: Graphics & Wayland Bridge

**Objetivo**: Exibir o conteúdo visual do app em uma janela nativa.

## Sub-tarefas
- [ ] **SurfaceFlinger Integration**:
    - Capturar o buffer final do SurfaceFlinger ou interceptar as chamadas do App para o compositor Android.
- [ ] **Wayland Client**:
    - Implementar um cliente Wayland em Rust (`smithay` ou `wayland-rs`).
    - Criar uma `wl_surface` para o app.
- [ ] **Buffer Sharing (DMA-BUF)**:
    - Passar os buffers de GPU do Android para o Wayland sem cópia (zero-copy).

## Referências
- [Waydroid Graphics implementation](https://github.com/waydroid/waydroid)
- [Wayland Protocol Explorer](https://wayland.app/protocols/)
- [Rust `wayland-rs`](https://github.com/Smithay/wayland-rs)
