# Task 043: Zero-Copy DMA-BUF Sharing

## Descrição Detalhada
Esta é a tarefa final de performance. Queremos passar os pixels do Android para o Wayland sem copiá-los na CPU.

## Detalhes de Implementação (Rust)
1.  Usar a extensão `linux-dmabuf` do Wayland.
2.  Importar o FD do buffer (obtido na Task 042) para o Wayland usando `zwp_linux_buffer_params_v1`.
3.  Anexar o buffer à `wl_surface` e realizar o `commit`.

## Resultado Esperado
O app Android aparece em uma janela de alta performance no desktop Linux, com aceleração de hardware.
