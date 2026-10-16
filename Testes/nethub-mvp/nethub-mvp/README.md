# NetHub (MVP)

Plataforma **terminal-first** para administração de infraestrutura. Este MVP implementa o
núcleo ponta a ponta e a base para expandir as demais funcionalidades.

## O que já funciona

| Caso de uso | Requisitos | Como usar |
|---|---|---|
| UC-01 Descobrir dispositivos | RF-01, RF-02, RNF-10 | `nethub descobrir 192.168.0.0/24` |
| UC-02 Consultar host e capacidades | RF-03, RN-04 | `nethub hosts` · `nethub host 192.168.0.10` |
| UC-03 Executar comando remoto | RF-04, RF-05, RN-01, RN-05 | `nethub exec 192.168.0.10 -- uptime` |
| UC-08 Histórico de operações | RF-09, RF-14, RNF-06 | `nethub historico` · `nethub historico --detalhe 3` |
| UC-09 Tags | RF-15 | `nethub tag add 192.168.0.10 web` |
| TUI | RF-10 | `nethub tui` |

Também: modelo de risco (Baixo/Médio/Alto/Crítico) com confirmação proporcional, dry-run obrigatório
para risco crítico, evidência para toda ação (inclusive canceladas e falhas) e **cadeia de hash**
nas evidências (`nethub verificar-auditoria` detecta adulteração).

## Requisitos

- Python 3.10+ (sem dependências externas; TUI usa `curses` da biblioteca padrão, Linux/macOS)
- `nmap` no PATH (descoberta) e cliente `ssh` do OpenSSH (execução remota)
- SSH por **chave/agente** (`BatchMode=yes`): o NetHub nunca recebe nem guarda senhas (RN-06)

## Instalação e uso

```bash
python -m nethub --help          # sem instalar
pip install -e .                 # ou instalar o comando `nethub`

nethub descobrir 192.168.0.0/24          # pede confirmação de autorização (ou use --autorizado)
nethub hosts
nethub host 192.168.0.10
nethub exec 192.168.0.10 --usuario admin -- df -h
nethub exec 192.168.0.10 --simular -- reboot              # só mostra o plano (dry-run)
nethub exec 192.168.0.10 --confirmar-ip 192.168.0.10 -- systemctl restart nginx
nethub tui --usuario admin
```

O banco fica em `~/.nethub/nethub.db` (ou `$NETHUB_DB`, ou `--db`), com permissão 600.

### Confirmação proporcional ao risco

| Risco | Exemplo | Confirmação |
|---|---|---|
| Baixo | `uptime`, `systemctl status x` | nenhuma |
| Médio | `touch`, redirecionamentos, comandos desconhecidos | `s/N` (ou `--yes`) |
| Alto | `systemctl restart`, `rm`, `sudo ...` | digitar o IP do alvo (ou `--confirmar-ip`) |
| Crítico | `rm -rf /`, `reboot`, `mkfs`, `curl \| sh` | dry-run + digitar o IP |

> A classificação é uma **heurística conservadora** (`domain/risk.py`), não uma barreira de segurança.

## Testes

```bash
python -m unittest discover -s tests -v
```

## Arquitetura (alinhada ao diagrama de classes)

```
nethub/
├── domain/          # entidades, enums, ações (Command), regras de risco, PORTAS (interfaces)
├── application/     # serviços: Discovery, Inventory, ActionExecutor, RiskPolicy, Evidence, History...
├── infrastructure/  # adaptadores: NmapAdapter, SshExecutor, SQLite (db + repositórios)
├── presentation/    # CLI (argparse), TUI (curses), formatação compartilhada
└── bootstrap.py     # raiz de composição (injeção de dependências)
tests/               # unittest + dublês (sem rede, sem nmap/ssh reais)
```

Dependências apontam para dentro: `presentation → application → domain ← infrastructure`.
Trocar Nmap/SSH/SQLite = escrever outro adaptador da porta correspondente (RNF-08, RNF-11).

## Como expandir (pontos de extensão)

| Funcionalidade | Onde plugar |
|---|---|
| Nova ação (health check, systemd, SFTP) | subclasse de `Acao` em `domain/actions.py` + `registro_padrao()`; fluxo na TUI em `NetHubTui.rodar_acao` |
| Capacidades SYSTEMD/SUDO/SFTP (RN-02) | sondagem ativa em `CapabilityDetector` (hoje só passiva: SSH = porta aberta) |
| Workflow com dry-run (UC-07) | `Workflow`/`EtapaWorkflow` no domínio + `WorkflowService` que reutiliza `ActionExecutor.simular/executar` |
| Validação pós-ação (RF-08) | método `validar()` em `Acao` chamado por `ActionExecutor` após `executar` |
| SFTP (UC-06) | nova porta `FileTransferPort` + `SftpAdapter`; checksum local/remoto |
| Auth por senha / Paramiko | novo `RemoteExecutorPort` + `CredentialStore` em memória (RN-06) |
| Exportar auditoria | consumir `EvidenceRepository.todas_em_ordem()` |

## Limitações conhecidas do MVP

- SSH `disponível` = porta aberta (não garante autenticação funcionando).
- Código de saída 255 do `ssh` é ambíguo (falha de conexão **ou** comando que saiu com 255).
- `StrictHostKeyChecking=accept-new` (confia na primeira conexão); ajustável em `SshExecutor`.
- Sem Windows (curses); a CLI funciona, a TUI não.
