# Task 033: Minimal System Services (Stubbing)

## Descrição Detalhada
Muitos apps crasham se não encontrarem o `ActivityManager` ou o `PackageManager` via Binder. Precisamos de um processo que responda a essas chamadas básicas.

## Detalhes de Implementação (Rust)
1.  Implementar um "Stub Service" usando o crate `rsbinder`.
2.  Registrar o serviço `activity` no `servicemanager`.
3.  Responder positivamente a métodos como `checkPermission` ou `getAppOpsService` com valores padrão "permitido".

## Objetivo
Enganar o App para que ele acredite que está rodando em um sistema Android completo, mesmo que o sistema seja apenas um esqueleto.
