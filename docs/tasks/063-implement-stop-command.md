# Task 063: Implement Stop Command

Status: pending
Priority: medium
Type: feature
Parent: 005-management-commands

## Description
A Task 005 (Management Commands) está 4/5 completa. O comando `stop` ainda está pendente. Falta implementar a capacidade de parar um app em execução.

## Estado Atual
- `doctor`: implementado
- `run`: implementado
- `shell`: implementado (com bug de chroot - Task 057)
- `reset`: implementado
- `logs`: implementado (sem --follow - Task 062)
- `stop`: **NÃO implementado**

## Implementação Proposta
1. Registrar PID do processo filho no prefix (arquivo `prefix/run.pid`)
2. Comando `stop` lê o PID e envia SIGTERM
3. Timeout + SIGKILL se não responder
4. Cleanup do PID file

## Todos
- [ ] Adicionar subcomando `stop <package>` ao CLI (clap)
- [ ] Salvar PID do child process em `{prefix}/run.pid` no comando `run`
- [ ] Implementar `Prefix::stop()` que lê PID e envia signal
- [ ] Implementar graceful shutdown (SIGTERM -> timeout -> SIGKILL)
- [ ] Limpar PID file após stop
- [ ] Tratar caso de PID stale (processo já morreu)
- [ ] Testar stop em processo rodando

## Critério de Aceite
- `cargo run -- stop <package>` para o app
- Mensagem clara se app não estiver rodando
- Cleanup de recursos após stop
