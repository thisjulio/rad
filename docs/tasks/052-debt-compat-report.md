# Task 052: [DEBT] Implement CompatReport Struct

Status: pending
Priority: medium
Type: technical-debt
Parent: 002-apk-inspector

## Description
A Task 002 marcou "Implement a CompatReport struct" como completo, mas essa struct não existe no código. Segundo AGENTS.md (Agent: APK Installer/Inspector), o sistema deve gerar um "compat report" indicando o que pode quebrar.

## Débito Identificado
Em `crates/apk/src/lib.rs`:
- ❌ Nenhuma struct `CompatReport` existe
- ❌ Não há análise de compatibilidade
- ❌ Não há warnings sobre dependências problemáticas

## Todos
- [ ] **Red**: Criar teste que valida geração de CompatReport
- [ ] **Green**: Criar struct `CompatReport` com campos:
  - `host_abi_compatible: bool` (se o host suporta as ABIs do app)
  - `warnings: Vec<String>` (ex: "Requires Play Services")
  - `blockers: Vec<String>` (ex: "No x86_64 lib for x86_64 host")
  - `compatibility_score: u8` (0-100)
- [ ] Implementar método `ApkInspector::generate_compat_report(&self, host_abi: Abi) -> CompatReport`
- [ ] Detectar dependências problemáticas:
  - Play Services (scan por `com.google.android.gms` no manifest)
  - Native ARM em host x86 (precisa de emulação com qemu-user)
  - Permissões privilegiadas (Camera, Location sem runtime)
- [ ] **Refactor**: Integrar compat report no comando `run`
- [ ] Adicionar flag `--force` para ignorar warnings

## Critério de Aceite
- `cargo run -- run test.apk` deve mostrar compat warnings
- Apps ARM-only em host x86_64 devem gerar warning
- Apps com Play Services devem gerar warning claro
- Testes devem validar pelo menos 5 cenários de incompatibilidade

## Output Esperado
```
✅ APK Metadata:
   📦 Package: com.example.app
   🏗️  ABIs: ["armeabi-v7a"]

⚠️  Compatibility Report:
   ❌ Host is x86_64 but app only has ARM libs (needs emulation)
   ⚠️  App requires Google Play Services (may not work)
   ℹ️  Compatibility Score: 40/100

Continue? [y/N]
```
