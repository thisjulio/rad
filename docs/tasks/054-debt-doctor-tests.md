# Task 054: [DEBT] Add Tests for Doctor Checks

Status: pending
Priority: medium
Type: technical-debt
Parent: 001-setup-workspace

## Description
O comando `doctor` está funcional e implementa 3 checks importantes, mas não possui testes unitários. Precisamos validar que os checks funcionam corretamente em diferentes cenários.

## Débito Identificado
Em `crates/core/src/doctor.rs`:
- ✅ Código funcional (check_binder, check_namespaces, check_overlayfs)
- ❌ Nenhum teste unitário
- ❌ Não há validação de mensagens de erro
- ❌ Não há teste para diferentes estados do sistema

## Todos
- [ ] **Red**: Criar teste `test_doctor_overlayfs_detection()`
- [ ] **Green**: Implementar teste que mocka leitura de `/proc/filesystems`
- [ ] Criar teste `test_doctor_user_namespaces_detection()`
- [ ] Criar teste `test_doctor_binder_detection()` com múltiplos cenários:
  - `/dev/binder` existe
  - `/dev/binderfs` existe
  - Nenhum existe (deve falhar)
- [ ] **Refactor**: Tornar funções de check testáveis (injeção de dependência para filesystem)
- [ ] Adicionar teste para `run_doctor()` completo
- [ ] Validar que mensagens de fix são exibidas corretamente

## Estratégia de Testes
Como os checks leem arquivos do sistema, precisamos:
1. Criar funções internas que aceitam um "filesystem reader" (trait)
2. Implementar um `MockFilesystem` para testes
3. Manter a função pública simples que usa filesystem real

## Exemplo de Refactor
```rust
// Antes (não testável)
fn check_overlayfs() -> DoctorIssue {
    let content = std::fs::read_to_string("/proc/filesystems").unwrap_or_default();
    // ...
}

// Depois (testável)
fn check_overlayfs_impl(filesystem_content: &str) -> DoctorIssue {
    // lógica de parse
}

pub fn check_overlayfs() -> DoctorIssue {
    let content = std::fs::read_to_string("/proc/filesystems").unwrap_or_default();
    check_overlayfs_impl(&content)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_overlayfs_detected() {
        let mock_fs = "nodev\toverlay\nnodev\ttmpfs\n";
        let result = check_overlayfs_impl(mock_fs);
        assert!(result.status);
    }
}
```

## Critério de Aceite
- `cargo test --package core` deve ter pelo menos 6 testes de doctor
- Todos os cenários (sucesso/falha) devem ser testados
- Mensagens de fix devem ser validadas
- Testes devem rodar sem acesso ao filesystem real
