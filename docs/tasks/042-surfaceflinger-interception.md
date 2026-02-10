# Task 042: SurfaceFlinger Interception

## Descrição Detalhada
No Android, o `SurfaceFlinger` compõe as janelas. Precisamos de um mecanismo para que os frames gerados pelo App cheguem ao nosso cliente Wayland.

## Fluxo TDD
- [x] **Red**: Teste que assevera a falha ao tentar capturar um frame de um buffer virtual inexistente.
- [x] **Green**: Implementar a lógica de configuração do buffer virtual e a interceptação via Gralloc/VirGL.
- [x] **Refactor**: Criar uma interface genérica de "FrameProvider" para suportar diferentes backends gráficos no futuro.

## Detalhes de Implementação (Rust)
1.  Configurar o App para renderizar em um buffer virtual.
2.  Usar o `Gralloc` do Android (geralmente Mesa/VirGL no nosso caso) para obter o file descriptor do buffer (DMA-BUF).
3.  Notificar o cliente Wayland quando um novo frame estiver pronto.
