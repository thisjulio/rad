# Task 042: SurfaceFlinger Interception

## Descrição Detalhada
No Android, o `SurfaceFlinger` compõe as janelas. Precisamos de um mecanismo para que os frames gerados pelo App cheguem ao nosso cliente Wayland.

## Estratégia
1.  Configurar o App para renderizar em um buffer virtual.
2.  Usar o `Gralloc` do Android (geralmente Mesa/VirGL no nosso caso) para obter o file descriptor do buffer (DMA-BUF).
3.  Notificar o cliente Wayland quando um novo frame estiver pronto.

## Referências
- [Mesa VirGL Documentation](https://virgil3d.github.io/)
- [Android Gralloc Architecture](https://source.android.com/docs/core/graphics/arch-gralloc)
