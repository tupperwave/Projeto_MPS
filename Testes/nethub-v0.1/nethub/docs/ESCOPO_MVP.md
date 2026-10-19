# NetHub v0.1 — escopo do MVP: o que foi mantido, adiado e alterado

Estado: **primeira metade do MVP funcional e testada** (CLI completa + TUI v0). Cobertura: 24 testes unitários,
39 testes ponta a ponta (`tests/e2e.sh`) e um teste da TUI em pseudo-terminal real, todos contra o laboratório local.

## 1. Requisitos funcionais (documento de elicitação)

| RF | Requisito | Estado v0.1 | Observação |
|----|-----------|-------------|------------|
| RF-01 | Descoberta Nmap → SQLite | ✅ Feito | `nmap -sT -oX -`; alvo validado; resultado incompleto é descartado; hosts do alvo não vistos viram `down` |
| RF-02 | Inventário (IP, SSH, hostname, portas, serviços, timestamps, tags) | ✅ Feito | + `facts` (uid, sudo, systemd, SO) coletados na inspeção |
| RF-03 | Menu contextual por capacidade | ✅ Feito | Mostra as compatíveis **e** as bloqueadas com o motivo |
| RF-04 | Modelo de risco + confirmação proporcional | ✅ Feito | BAIXO/MÉDIO/ALTO/CRÍTICO; risco de `exec` calculado pelo comando (heurística) |
| RF-05 | Execução remota não interativa + auditoria | ✅ Feito | Script em base64 (sem injeção de quoting), timeout, exit code, stdout/stderr |
| RF-07 | Workflow com dry-run | ✅ Feito | 2 workflows embutidos; dry-run reverifica capacidades ao vivo e mostra os comandos |
| RF-08 | Validação pós-ação | ✅ Feito | Reinício só é "sucesso" se ativo **e** PID/instante de ativação mudou; transferência só com SHA-256 igual |
| RF-09 | Registro de evidências | ✅ Feito | Hash encadeado (tamper-evident) + `audit verify` |
| RF-10 | Interface TUI | 🟡 v0 | Hosts, contexto, menu, confirmação, resultados, histórico. Faltam: tela de serviços dedicada, workflows com parâmetros guiados melhores, temas |
| RF-11 | Health check | ✅ Feito | Disco, memória, carga, uptime, unidades com falha; métrica ausente = INDISPONÍVEL |
| RF-12 | Gestão de serviços systemd | ✅ Feito | list/status/restart com verificação |
| RF-13 | SFTP com checksum | ✅ Feito | Upload/download; sobrescrita registrada; divergência = falha |
| RF-14 | Histórico consultável | ✅ Feito | Filtros por host, operação, status, texto, id |
| RF-15 | Tags | ✅ Feito | CLI e TUI |

## 2. Regras de negócio

| RN | Estado | Como foi implementado |
|----|--------|-----------------------|
| RN-01 confirmação explícita para ALTO/CRÍTICO | ✅ | Digitar nome/IP do alvo; `--yes` **não** basta |
| RN-02 "autorizado" só com sudo | ✅ | `sudo -n true` testado de verdade (ou uid 0) |
| RN-03 toda ação gera evidência | ✅ | Inclusive bloqueadas, canceladas, dry-runs e falhas |
| RN-04 ações só se compatíveis | ✅ | `Guard` central impede a execução mesmo que a UI tente |
| RN-05 dry-run obrigatório para CRÍTICO | ✅ | O dry-run fica na trilha e vale 15 min **para aquele plano exato** (host+comando) |
| RN-06 credenciais não persistidas | ✅ | Só ssh-agent/chaves; o banco guarda usuário, porta e *caminho* da chave |
| RN-07 agentless, local-first | ✅ | Usa `ssh`, `sftp` e `nmap` do sistema; nada instalado nos hosts |

## 3. Casos de uso

UC-01 a UC-09: todos cobertos (CLI e TUI, exceto UC-09 só com tag *adicionar* na TUI).
Exceções previstas nos UCs (Nmap ausente, timeout, cancelamento, serviço inexistente, checksum divergente,
métrica indisponível, capacidade desconhecida) têm teste.

## 4. Adiado para as próximas iterações

| Item | Por quê | Iteração |
|------|---------|----------|
| Terminal interativo (PTY) | Complexidade alta; `exec` não interativo cobre a tese | v0.4 (suspender a TUI e entregar o terminal ao `ssh`) |
| Workflows definidos pelo usuário (arquivo TOML/JSON) | Os 2 embutidos provam o motor | v0.3 |
| Rollback/compensação | Não existe rollback genérico (ex.: `apt upgrade`). A política `COMPENSATE` está reservada e tratada como STOP | v0.5 (compensação por etapa declarada) |
| Windows, SNMP | Fora do escopo do TCC | Fora |
| Smart Collections, topologia, grafo de dependências, Health Score | Não provam a tese | v0.5+ |
| Agendamento | Idem | v0.6 |
| Multiusuário / RBAC | Idem | Fora |

## 5. Decisões e desvios em relação à proposta

1. **Sem Tokio**: threads padrão + canal bastam (a TUI nunca bloqueia) e reduzem dependências. O contrato `INetworkScanner`/`IRemoteExecutor` permite trocar depois.
2. **Quatro níveis de capacidade** (`UNKNOWN < REACHABLE < AUTHENTICATED < AUTHORIZED`) em vez de três: o diagrama de classes não tinha o "alcançável" que a proposta exige.
3. **Estado `BLOCKED`** em `OperationStatus`: tentativas barradas por capacidade/política são evidência.
4. **Risco de `cmd.exec` é heurística** (lista de somente-leitura + padrões perigosos; na dúvida sobe o risco, nunca desce). Sempre mostra os motivos. É conservadora, **não é um sandbox**: o operador confirma o que será executado.
5. **Download = MÉDIO** (como na tabela da proposta), mesmo sendo leitura: copia dados para fora do host.
6. **TOFU de host key** (`StrictHostKeyChecking=accept-new`): primeira conexão confia, mudança posterior é recusada (`FailureKind::HostKey`).
7. **Truncamento da trilha**: a cadeia detecta edição, remoção no meio e reordenação, mas não a remoção dos *últimos* registros. Use `nethub audit head` e guarde o hash fora da máquina.
8. **systemd testado com emulação** (`lab/fake-systemctl`): o contêiner de desenvolvimento não tinha systemd. Os formatos usados (`systemctl show -p`, `list-units --plain`, `is-system-running`) são os reais, mas **valide em uma VM Ubuntu/Debian de verdade** antes de afirmar resultados no TCC.

## 6. Limitações conhecidas

- Health check e pré-atualização validados em Ubuntu 24.04; outras distros não foram testadas (pré-atualização exige `apt`).
- Descoberta só IPv4, TCP connect (sem OS fingerprint nem ARP/MAC fora da LAN local).
- A TUI v0 não tem rolagem horizontal nem layout adaptativo para terminais menores que ~100 colunas.
