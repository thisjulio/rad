# Task 022: Binderfs Instance Setup (The IPC Heart)

## Descrição Detalhada
Para que o Android funcione, ele precisa dos devices `/dev/binder`, `/dev/hwbinder` e `/dev/vndbinder`. No nosso modelo isolado, não podemos usar os devices do host. Devemos montar uma instância privada do `binderfs` dentro do Mount Namespace do App.

## Detalhes de Implementação (Rust)
1.  **Mount**: Dentro do novo namespace, executar `mount("binder", "/dev/binderfs", "binder", MS_NODEV | MS_NOEXEC | MS_NOSUID, None)`.
2.  **Alocação de Devices**:
    - Abrir `/dev/binderfs/binder-control`.
    - Usar `ioctl` com o comando `BINDER_CTL_ADD` passando uma estrutura `binderfs_device` contendo o nome (ex: "binder").
    - Repetir para "hwbinder" e "vndbinder".
3.  **Links**: Criar links simbólicos ou bind mounts para que o app encontre esses devices em `/dev/`.

## Desafios Técnicos
Lidar com `unsafe` blocks do Rust para chamadas `ioctl` e garantir que a estrutura de memória esteja perfeitamente alinhada com o que o kernel Linux espera (`#[repr(C)]`).

## Referências
- [rsbinder (Pure Rust Binder Implementation)](https://github.com/neofelis/rsbinder)
- [Linux Kernel Header: binderfs.h](https://github.com/torvalds/linux/blob/master/include/uapi/linux/android/binderfs.h)
