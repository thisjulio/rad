# Task 061: [DEBT] Implement Real Wayland Protocol Integration

Status: pending
Priority: medium
Type: technical-debt
Parent: 042-surfaceflinger-interception, 043-zero-copy-dma-buf

## Description
As Tasks 042 e 043 estão marcadas como completas `[x]`, mas a implementação real **não usa nenhuma API Wayland**. O crate `wayland` tem buffer management em memória mas zero integração com o protocolo Wayland.

## Discrepância Documentação vs Código

### Task 042 diz:
> "Implementar a lógica de configuração do buffer virtual e a interceptação via Gralloc/VirGL"
- **Realidade**: Apenas `VirtualBuffer` (wrapper de OwnedFd) e `FrameProvider` trait SEM implementação

### Task 043 diz:
> "Implementar o uso da extensão linux-dmabuf e o commit do buffer na superfície"
- **Realidade**: `DmabufBuffer` e `SurfaceDmabufManager` existem, mas:
  - NÃO usam `zwp_linux_buffer_params_v1`
  - NÃO criam `wl_buffer` a partir do DMA-BUF fd
  - NÃO fazem `wl_surface.attach()` ou `wl_surface.commit()`
  - As deps `wayland-client` e `wayland-protocols` no `Cargo.toml` **nunca são importadas no código**

### `FrameProvider` trait:
- Definido em `lib.rs:71-75` com 3 métodos
- **Zero implementações** em todo o codebase

## Impacto
Sem integração Wayland real, frames renderizados pelo Android nunca chegarão ao compositor do host. Isso impede qualquer aplicação com GUI.

## Todos
- [ ] Atualizar status das Tasks 042 e 043 para refletir implementação parcial
- [ ] Implementar `WaylandConnection` real usando `wayland-client` 0.29
- [ ] Implementar import de DMA-BUF fd via `zwp_linux_dmabuf_v1`
- [ ] Implementar `wl_surface` attach + commit
- [ ] Criar pelo menos uma implementação de `FrameProvider`
- [ ] Implementar event loop de sincronização compositor/app
- [ ] Remover `VirtualBuffer` duplicado (usar apenas `DmabufBuffer`)

## Critério de Aceite
- Conexão Wayland real estabelecida
- Buffer DMA-BUF importado como `wl_buffer`
- Frame visível no compositor Wayland do host
