# Task 058: [DEBT] Doctor Uses Weak Namespace Check

Status: pending
Priority: medium
Type: technical-debt
Parent: 001-setup-workspace

## Description
O `core::doctor::check_namespaces()` usa uma verificação fraca (apenas `Path::new("/proc/self/ns/user").exists()`), mas o crate `sandbox::doctor` tem uma verificação real que faz fork() + unshare(CLONE_NEWUSER).

## Problema
- `crates/core/src/doctor.rs:78`: apenas verifica se o arquivo existe no procfs
- `crates/sandbox/src/doctor.rs:67-103`: faz fork() + teste real de criação de namespace
- A verificação do sandbox é **superior** mas **não é usada** pelo core doctor

## Impacto
Um sistema pode ter `/proc/self/ns/user` mas ter namespaces desabilitados via sysctl (`kernel.unprivileged_userns_clone=0`). A verificação fraca daria falso positivo.

## Todos
- [ ] Alterar `core::doctor::check_namespaces()` para usar `sandbox::doctor::check_user_namespaces()`
- [ ] Remover verificação duplicada baseada apenas em path
- [ ] Adicionar teste para cenário onde path existe mas namespace está desabilitado
- [ ] Manter mensagens de fix/diagnóstico atuais

## Critério de Aceite
- `doctor` usa fork-based namespace check
- Falsos positivos eliminados
- `cargo test` passa
