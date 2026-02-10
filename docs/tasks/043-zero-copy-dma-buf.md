# Task 043: Zero-Copy DMA-BUF Sharing

## Descrição Detalhada
Esta é a tarefa final de performance. Queremos passar os pixels do Android para o Wayland sem copiá-los na CPU.

## Fluxo TDD
- [x] **Red**: Teste que assevera a falha ao tentar importar um FD inválido como buffer Wayland.
- [x] **Green**: Implementar o uso da extensão `linux-dmabuf` e o commit do buffer na superfície.
- [x] **Refactor**: Garantir a sincronização correta entre o release do buffer pelo compositor e a escrita pelo app.

## Detalhes de Implementação (Rust)
1.  Usar a extensão `linux-dmabuf` do Wayland.
2.  Importar o FD do buffer (obtido na Task 042) para o Wayland usando `zwp_linux_buffer_params_v1`.
3.  Anexar o buffer à `wl_surface` e realizar o `commit`.
