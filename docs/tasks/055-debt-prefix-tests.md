# Task 055: [DEBT] Add Tests for Prefix Creation

Status: pending
Priority: medium
Type: technical-debt
Parent: 003-runtime-orchestrator

## Description
A struct `Prefix` está funcional e cria a estrutura de diretórios corretamente, mas não possui testes unitários validando seu comportamento.

## Débito Identificado
Em `crates/core/src/prefix.rs`:
- ✅ Código funcional (Prefix::new, initialize)
- ❌ Nenhum teste unitário
- ❌ Não há validação de estrutura criada
- ❌ Não há teste de idempotência (chamar initialize() duas vezes)

## Todos
- [ ] **Red**: Criar teste `test_prefix_initialization()`
- [ ] **Green**: Validar que todos os 8 diretórios são criados:
  - system, data, data/app, data/data, dev, proc, sys, tmp, apex
- [ ] Criar teste `test_prefix_idempotent()` - chamar initialize() 2x não deve falhar
- [ ] Criar teste `test_prefix_permissions()` - verificar permissões dos diretórios
- [ ] **Refactor**: Adicionar método `Prefix::exists() -> bool`
- [ ] Adicionar método `Prefix::clean()` para remover prefix
- [ ] Testar comportamento com path inválido

## Exemplo de Teste
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_prefix_creates_android_structure() {
        let tmp = tempdir().unwrap();
        let prefix = Prefix::new(tmp.path().join("test-prefix"));
        
        prefix.initialize().unwrap();
        
        assert!(prefix.root.join("system").exists());
        assert!(prefix.root.join("data/app").exists());
        assert!(prefix.root.join("data/data").exists());
        assert!(prefix.root.join("dev").exists());
        assert!(prefix.root.join("proc").exists());
        assert!(prefix.root.join("sys").exists());
        assert!(prefix.root.join("tmp").exists());
        assert!(prefix.root.join("apex").exists());
    }

    #[test]
    fn test_prefix_initialize_idempotent() {
        let tmp = tempdir().unwrap();
        let prefix = Prefix::new(tmp.path().join("test-prefix"));
        
        prefix.initialize().unwrap();
        prefix.initialize().unwrap(); // não deve falhar
        
        assert!(prefix.root.join("system").exists());
    }
}
```

## Dependências Adicionais
Adicionar ao `crates/core/Cargo.toml`:
```toml
[dev-dependencies]
tempfile = "3.8"
```

## Critério de Aceite
- `cargo test --package core` deve ter pelo menos 3 testes de Prefix
- Testes devem usar diretórios temporários (tempfile)
- Testes devem limpar após execução
- Validar estrutura completa de diretórios
