# Task 033: Minimal System Services (Stubbing)

## Descrição Detalhada
Muitos apps crasham se não encontrarem o `ActivityManager` ou o `PackageManager` via Binder. Precisamos de um processo que responda a essas chamadas básicas.

## Fluxo TDD
- [ ] **Red**: Teste que tenta realizar uma chamada Binder para um serviço mockado e recebe erro de "service not found".
- [ ] **Green**: Implementar o registro do serviço usando `rsbinder` e fornecer uma resposta básica.
- [ ] **Refactor**: Implementar um sistema de despacho de chamadas para facilitar a adição de novos stubs.

## Detalhes de Implementação (Rust)
1.  Implementar um "Stub Service" usando o crate `rsbinder`.
2.  Registrar o serviço `activity` no `servicemanager`.
3.  Responder positivamente a métodos como `checkPermission` ou `getAppOpsService` com valores padrão "permitido".
