# Task 031: AOSP Payload Strategy

## Descrição Detalhada
O "Payload" é o conjunto de bibliotecas e executáveis do Android (AOSP) que o `run-android-app` precisa para funcionar. Como o objetivo é "single-binary", precisamos de uma estratégia para gerenciar esses arquivos.

## Opções de Implementação (Rust)
1.  **Embed**: Usar a macro `include_bytes!` para embutir um arquivo `.tar.zst` dentro do binário.
2.  **External Download**: Baixar a runtime na primeira execução (estilo `rustup`).
3.  **Local Build**: Gerar a partir de um script se o usuário tiver as ferramentas.

*Decisão*: Para o MVP v0, usaremos **Embed** (embutido) para garantir que "um binário" realmente funcione sem internet.

## Detalhes
- O payload deve conter o `linker64` (Bionic), `libart.so`, e os jars do framework.
- Deve ser extraído para um diretório de cache em `~/.cache/run-android-app/payloads/<version>`.
