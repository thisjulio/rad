# Task 041: Wayland Connection & Registry (Rust)

## Descrição Detalhada
Para exibir a interface do Android como uma janela nativa, o `run-android-app` deve atuar como um cliente Wayland. Esta tarefa foca no setup inicial da conexão com o compositor do host (ex: GNOME/Mutter, KDE/KWin, Sway).

## Fluxo TDD
- [ ] **Red**: Teste unitário que assevera a falha na conexão quando `WAYLAND_DISPLAY` não está presente ou é inválido.
- [ ] **Green**: Implementar a conexão usando `wayland-client` e o processamento inicial do registry.
- [ ] **Refactor**: Encapsular a conexão Wayland em uma struct com gerenciamento de estado assíncrono.

## Detalhes de Implementação (Rust)
1.  Usar o crate `wayland-client`.
2.  Conectar ao socket (geralmente via variável de ambiente `WAYLAND_DISPLAY`).
3.  Implementar o `wl_registry` para listar as capacidades do servidor:
    - `wl_compositor`: Para criar a superfície.
    - `wl_shm` ou `linux_dmabuf`: Para gerenciar os buffers de vídeo.
    - `xdg_wm_base`: Para criar a janela decorada no desktop.
