# Task 062: Implement --follow Flag for Logs Command

Status: pending
Priority: low
Type: bug
Parent: 005-management-commands

## Description
O flag `--follow` / `-f` no comando `logs` é aceito pelo parser (clap) mas **completamente ignorado** na implementação.

## Problema
Em `crates/cli/src/main.rs:119`:
```rust
Command::Logs { package, follow: _ } => {
    //                           ^^^ ignorado!
```

O `_` descarta o valor do flag. Comportamento deveria ser equivalente a `tail -f`.

## Implementação Proposta
- Usar `notify` crate para watch no arquivo de log
- Ou polling simples com `tokio::time::interval`
- Imprimir novas linhas conforme são escritas em `logs/app.log`

## Todos
- [ ] Implementar leitura contínua do arquivo de log quando `--follow` é true
- [ ] Usar `Ctrl+C` para sair do follow mode
- [ ] Testar com log sendo escrito em paralelo

## Critério de Aceite
- `cargo run -- logs -f com.example.app` mostra logs em tempo real
- `Ctrl+C` encerra o follow
