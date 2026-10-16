# NetHub v0.1

Plataforma **terminal-first** para administrar a infraestrutura de pequenos ambientes a partir de
**capacidades verificadas**, **risco explícito** e **evidência de execução**.

> Descoberta → inventário → *o que posso fazer neste host (provado)* → ação com risco → validação → evidência.

Projeto: André Luiz Rodrigues Castro da Nóbrega · Marcos José Edu Ekua Ayingono · Lucas Martins de Lima Oliveira

## O que é diferente

| | NetHub | Ansible | Zabbix | RMMs |
|---|---|---|---|---|
| Distingue alcançável / autenticado / autorizado (sudo provado) | ✅ | ❌ | ❌ | parcial |
| Mostra só o que dá para fazer **e por que o resto está bloqueado** | ✅ | ❌ | ❌ | ❌ |
| Confirmação proporcional ao risco + dry-run obrigatório para crítico | ✅ | ❌ | ❌ | ❌ |
| Valida o resultado (reinício só é sucesso se o serviço realmente reiniciou) | ✅ | manual | ❌ | ❌ |
| Evidência com hash encadeado (adulteração detectável) | ✅ | ❌ | ❌ | ❌ |
| Agentless, local, um binário | ✅ | ✅ | ❌ | ❌ |

*(Tabela de posicionamento a validar na pesquisa de trabalhos relacionados — veja `docs/AVALIACAO.md`.)*

## Requisitos

`ssh`, `sftp` e `nmap` no PATH (Debian/Ubuntu: `sudo apt install openssh-client nmap`). Rust ≥ 1.85 para compilar
(testado com 1.85). Nos hosts gerenciados: Linux com OpenSSH; `systemd`/`sudo -n` para as ações de serviço.

```bash
cargo build --release          # binário em target/release/nethub
./target/release/nethub        # abre a TUI   (ou: nethub --help)
```

## Uso rápido (CLI)

```bash
nethub scan 192.168.1.0/24                          # descoberta (pede confirmação: varredura exige autorização)
nethub hosts
nethub host set srv01 --user admin --key ~/.ssh/id_ed25519     # só dados de conexão; senhas nunca são guardadas
nethub onboard srv01 [--copy-id]                    # guia para autenticar por chave
nethub inspect srv01                                # PROVA: autenticação, sudo, systemd, SFTP
nethub actions srv01                                # o que é possível — e o que está bloqueado, com o motivo
nethub health srv01
nethub exec srv01 -- df -h                          # risco calculado pelo comando
nethub exec srv01 --dry-run -- sudo apt upgrade -y  # mostra plano, risco e comandos; registra o dry-run
nethub service restart srv01 nginx                  # reinicia e valida (ativo + PID/instante mudou)
nethub upload srv01 ./nginx.conf /tmp/nginx.conf    # SFTP + SHA-256 nas duas pontas
nethub workflow dry-run restart-service-validated srv01 --param service=nginx
nethub workflow run pre-update-check srv01          # GO / NO-GO antes de atualizar (Debian/Ubuntu)
nethub history --host srv01 --op service            # evidências;  --id N mostra o registro completo
nethub audit verify                                 # recalcula a cadeia de hashes
```

Confirmação por risco: **BAIXO** executa · **MÉDIO** `--yes` (ou `[s/N]` no terminal) · **ALTO** `--confirm <nome-ou-IP>` ·
**CRÍTICO** `--dry-run` antes **e** `--confirm <nome-ou-IP>`. Em terminal interativo o NetHub mostra o plano e pergunta.
Códigos de saída: 0 ok · 1 operação falhou · 2 entrada inválida · 3 bloqueado por capacidade · 4 falta confirmação/dry-run.

## TUI

`1` Hosts · `2` Histórico · `3` Ajuda. Em Hosts: `↑↓` navegar, `i` provar capacidades, `h` saúde, `a`/`Enter` menu de ações,
`t` tag, `d` descobrir, `r` atualizar, `q` sair. Em Histórico: `Enter` detalhes/evidência, `v` verificar integridade, `/` filtrar.

## Testar (laboratório local)

> Use **somente em VM/contêiner descartável**: o laboratório cria usuários e `sudo` sem senha.

```bash
cargo test                    # 24 testes unitários
sudo bash tests/e2e.sh        # sobe o lab e roda 39 verificações ponta a ponta
sudo bash lab/teardown_lab.sh # remove o lab
```

O lab cria 4 "hosts" em loopback (127.0.0.1–4): autorizado, só autenticado, sem chave e sem SSH. O systemd é **emulado**
(`lab/fake-systemctl`); valide também em VMs reais.

## Arquitetura (hexagonal, conforme o diagrama de classes)

```
CLI / TUI ──► Core (despacho)
                └─ services: Discovery · Capability · ActionCatalog · RiskPolicy · Guard · Plan(dry-run)
                             RemoteCommand · Health · Systemd · FileTransfer · Workflow · Audit · Inventory
                      └─ ports (traits) ──► adapters: NmapScanner · OpenSSHExecutor · SFTPAdapter · SQLite
```
Toda ação passa pelo `Guard` (capacidade provada + confirmação), nunca direto pela interface. Dados: `~/.nethub/nethub.db`
(permissão 700; contém inventário e trilha de auditoria — proteja como dado sensível).

## Roteiro incremental

| Versão | Entrega |
|---|---|
| **v0.1 (esta)** | Descoberta, capacidades em níveis, risco, exec, health, systemd, SFTP, 2 workflows com dry-run, evidência encadeada, CLI completa, TUI v0 |
| v0.2 | Validação em VMs reais (Ubuntu/Debian), ajustes de parsing; TUI: tela de serviços, workflows guiados, rolagem horizontal |
| v0.3 | Workflows definidos em arquivo (TOML), `--json` na CLI para automação/medição, exportação de relatório |
| v0.4 | Terminal interativo (suspende a TUI e entrega o terminal ao `ssh`), onboarding guiado na TUI |
| v0.5 | Compensação por etapa declarada (rollback honesto), dependências declaradas entre serviços |
| v0.6 | Experimento comparativo CLI × NetHub (SUS, tempo, erros) e capítulo de resultados |

Detalhes do que foi mantido/adiado/alterado: `docs/ESCOPO_MVP.md`. Guia de avaliação do diferencial: `docs/AVALIACAO.md`.
