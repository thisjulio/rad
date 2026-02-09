# Task 031: AOSP Payload Strategy

## Descrição Detalhada
O "Payload" é o conjunto de bibliotecas e executáveis do Android (AOSP) que o `run-android-app` precisa para funcionar. Como o objetivo é "single-binary", precisamos de uma estratégia para gerenciar esses arquivos.

## Fluxo TDD
- [ ] **Red**: Teste unitário que tenta ler um arquivo mock do payload embutido e falha se não houver lógica de extração.
- [ ] **Green**: Implementar `include_bytes!` e a lógica de descompressão/extração para o cache.
- [ ] **Refactor**: Adicionar verificação de integridade (checksum) do payload extraído.

## Detalhes de Implementação (Rust)
1.  **Embed**: Usar a macro `include_bytes!` para embutir um arquivo `.tar.zst` dentro do binário.
2.  **Extração**: Implementar lógica para extrair apenas se necessário (check de versão/hash).
3.  O payload deve conter o `linker64` (Bionic), `libart.so`, e os jars do framework.
