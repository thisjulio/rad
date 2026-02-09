# Task 002: Check Binderfs Availability

## Descrição Detalhada
O Android utiliza o IPC Binder para quase toda a comunicação entre o App e o Sistema. Tradicionalmente, isso dependia de `/dev/binder` (um device node fixo). O **binderfs** é a evolução que permite criar instâncias isoladas do Binder para cada container/prefix.

Esta tarefa deve validar se o host suporta `binderfs`, que é o requisito "non-negotiable" para rodar múltiplos apps isolados.

## Detalhes de Implementação (Rust)
1.  **Checar `/proc/filesystems`**: Verificar se a string `binder` está presente na lista de filesystems suportados pelo kernel.
2.  **Validar `/dev/binderfs/binder-control`**: Este é o ponto de controle para criar novos devices. O "doctor" deve reportar se este arquivo existe e se o usuário atual tem permissão de leitura/escrita.
3.  **Kernel Config**: Se possível, tentar ler `/proc/config.gz` ou `/boot/config-$(uname -r)` procurando por `CONFIG_ANDROID_BINDERFS=y`.

## Referências
- [Kernel Admin Guide: Binderfs](https://www.kernel.org/doc/html/latest/admin-guide/binderfs.html)
- [Brauner's Blog: The Android Binderfs](https://brauner.io/2019/01/09/android-binderfs.html)
