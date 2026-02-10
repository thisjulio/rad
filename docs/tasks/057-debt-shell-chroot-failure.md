# Task 057: [DEBT] Fix Shell Command Failure After Chroot

Status: pending
Priority: high
Type: bug
Parent: 006-interactive-shell

## Description
O comando `shell` falha com `ENOENT` após entrar no sandbox. O problema é que a resolução do shell é feita antes do chroot, mas o path resolvido não existe dentro do chroot.

## Reprodução
```bash
cargo run -- run --force crates/apk/test_data/real.apk
cargo run -- shell org.fdroid.fdroid
# Output: Sandbox setup failed: ENOENT: No such file or directory
```

## Análise Técnica

### Fluxo atual (prefix.rs):
1. `resolve_shell_command()` resolve para `/bin/sh` (caminho do HOST)
2. `run_in_sandbox()` faz fork -> enter_namespaces -> setup_mounts -> chroot
3. Após chroot, o processo tenta exec(`/bin/sh`) 
4. `/bin/sh` **não existe dentro do prefix** - apenas no payload

### Problema raiz:
- `setup_sandbox_mounts()` faz bind mount de `payload/system` em `{prefix}/system/`
- `setup_sandbox_mounts()` faz bind mount de `/bin` do host em `{prefix}/bin/`
- MAS: o bind mount de `/bin` acontece DENTRO do mount namespace
- Após `chroot()`, o filesystem é isolado ao prefix
- A busca de shell deveria considerar os caminhos DENTRO do chroot, não do host

### Possíveis soluções:
1. Resolver shell como path relativo ao prefix/payload (ex: `/system/bin/sh`)
2. Garantir que `busybox sh` do payload está acessível em `/system/bin/sh`
3. Ajustar o bind mount de `/bin` para acontecer antes da resolução

## Todos
- [ ] Investigar por que bind mount de `/bin` falha silenciosamente (ENOENT não é propagado)
- [ ] Alterar `resolve_shell_command()` para usar paths relativos ao chroot
- [ ] Verificar se `payload/system/bin/sh` (symlink para busybox) é acessível
- [ ] Testar que `shell` funciona após chroot com payload montado
- [ ] Adicionar teste de integração para shell no sandbox

## Critério de Aceite
- `cargo run -- shell <package>` abre um shell interativo
- O shell usa busybox do payload quando disponível
- Erro claro quando nenhum shell é encontrado no sandbox
