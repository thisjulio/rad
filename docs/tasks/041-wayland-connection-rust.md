# Task 041: Wayland Connection & Registry (Rust)

## Descrição Detalhada
Para exibir a interface do Android como uma janela nativa, o `run-android-app` deve atuar como um cliente Wayland. Esta tarefa foca no setup inicial da conexão com o compositor do host (ex: GNOME/Mutter, KDE/KWin, Sway).

## Detalhes de Implementação (Rust)
1.  Usar o crate `wayland-client`.
2.  Conectar ao socket (geralmente via variável de ambiente `WAYLAND_DISPLAY`).
3.  Implementar o `wl_registry` para listar as capacidades do servidor:
    - `wl_compositor`: Para criar a superfície.
    - `wl_shm` ou `linux_dmabuf`: Para gerenciar os buffers de vídeo.
    - `xdg_wm_base`: Para criar a janela decorada no desktop.

## Por que Wayland e não X11?
Wayland permite o compartilhamento direto de buffers de GPU (DMA-BUF) entre o container Android e o host de forma muito mais eficiente e moderna, o que é vital para performance de vídeo.

## Referências
- [Wayland-rs Guide](https://github.com/Smithay/wayland-rs)
- [Wayland Protocol Explorer](https://wayland.app/protocols/wayland)
