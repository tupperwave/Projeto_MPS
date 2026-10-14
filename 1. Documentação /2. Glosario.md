##Glossário 

### Produto
| Termo | 
| NET-HUB | Ferramenta TUI terminal-first para descoberta, inventário, execução de ações e auditoria em infraestrutura. |
| Terminal-first | Abordagem em que a interface principal é o terminal; o núcleo pode ser reutilizado por CLI, API ou outra interface no futuro. |
| TUI | Interface interativa de terminal, com painéis, menus, tabelas e navegação por teclado. |
| Agentless | Não exige agente próprio nos hosts administrados; pode usar ferramentas externas, como SSH e Nmap. |
| Local por padrão | Aplicação e dados operacionais residem na máquina do operador, salvo decisão explícita em contrário. |
| Operador | Pessoa que usa o NET-HUB; responsável por definir escopo autorizado e revisar riscos. |
| Binário único | Meta de entregar o programa principal como executável; não significa ausência de dependências externas. |

### Inventário

| Termo | 
|---|---|
| Descoberta de rede | Identificação de hosts e serviços em escopo definido; no MVP, usa Nmap e leitura de XML. |
| CIDR | Notação de bloco IP, como `192.168.1.0/24`, usada para delimitar o escopo autorizado. |
| Host | Dispositivo identificado na rede, como servidor, computador, NAS ou equipamento de rede. |
| Serviço de rede | Aplicação exposta por um host, como SSH ou HTTP. |
| Porta de rede | Ponto lógico de comunicação; porta aberta indica acessibilidade, não autorização. |
| Nmap | Ferramenta de descoberta e análise de rede usada no MVP. |
| Inventário | Registro local de hosts, serviços, tags e última observação; pode ficar desatualizado. |
| Tag | Rótulo manual para organizar ou filtrar hosts, como `produção`, `lab` ou `backup`. |
| Adaptador | Componente que integra o núcleo a ferramentas externas, como Nmap, SSH e SFTP. |
| Menu contextual de ações | Menu que exibe ações conforme capacidades e pré-requisitos do host selecionado. |
| Health check | Verificações objetivas de disco, memória, carga e serviços systemd em hosts compatíveis. |
| Serviço systemd | Unidade gerenciada pelo systemd; no MVP, consulta e restart em Debian/Ubuntu compatíveis. |
| SSH | Protocolo de acesso remoto seguro; mecanismo inicial para autenticação e operação remota. |
| SFTP | Protocolo de transferência de arquivos sobre SSH, com checksum quando aplicável. |
| OpenSSH | Implementação usada como dependência externa para SSH e SFTP. |
| SQLite | Banco relacional local para inventário, tags e registros operacionais do protótipo. |

### 2.3 Capacidades, risco e segurança

| Termo |
|---|---|
| Capability | Ação que o NET-HUB entende ser possível em um host, dadas evidências, credenciais e privilégios. |
| Detecção de capacidades | Processo que converte evidências técnicas em ações apresentáveis, sem inferir permissão sem verificação. |
| Alcançável | Serviço ou endpoint acessível pela rede; não comprova autenticação nem autorização. |
| Autenticação | Verificação de identidade; uma sessão SSH válida não implica autorização irrestrita. |
| Autorização | Permissão para executar uma ação específica sobre um recurso determinado. |
| Privilégio | Nível de permissão necessário à ação; possuir sudo não torna toda ação segura. |
| Credencial | Mecanismo de identidade, como chave SSH, certificado ou senha; evitar persistir segredos em texto claro. |
| Risco por capability | Classificação de impacto da ação: baixo, médio, alto ou crítico; define a confirmação exigida. |
| Pré-condição | Condição necessária antes de uma ação, como host acessível, autenticação válida ou serviço identificado. |
| Auditoria | Registro e consulta de ações e eventos para reconstruir o que foi feito, onde, quando e com qual resultado. |
| Evidência | Dados do resultado de uma ação: status, código de saída, saída, timestamp e hash ou checksum. |
| Histórico de operações | Lista cronológica de ações tentadas ou concluídas, distinguindo solicitação, falha e validação. |
| Hash criptográfico | Resumo para verificar integridade; não garante imutabilidade absoluta nem origem. |
| Checksum | Valor usado para verificar se o conteúdo transferido corresponde ao esperado. |
| Timestamp | Marca temporal de evento, descoberta ou execução, com formato e fuso documentados. |

### 2.4 Operação e workflow

| Termo |
|---|---|
| Ação administrativa | Operação que consulta, altera ou controla recurso de TI; deve ter alvo, risco e resultado registrável. |
| Alvo | Host, serviço, arquivo ou recurso sobre o qual a ação será executada. |
| Execução remota | Operação executada em host diferente da máquina do NET-HUB; no MVP, via SSH. |
| Comando não interativo | Comando remoto sem perguntas manuais; exige parâmetros, timeout e tratamento de erro definidos. |
| Dry-run | Simulação que apresenta etapas sem alterar o alvo; só é confiável se bloquear alterações reais. |
| Validação pós-ação | Verificação posterior para confirmar se o estado esperado foi atingido. |
| Workflow | Sequência controlada de etapas com alvo, pré-condições, confirmações e validação final. |
| Rollback | Tentativa de retornar ao estado anterior; não deve ser prometido sem mecanismo testado. |
| Timeout | Limite de tempo para descoberta, conexão ou comando remoto. |
| Código de saída | Número retornado pelo processo; zero geralmente indica sucesso, mas depende do comando. |
| Status da operação | Estado registrado: planejada, em execução, concluída, falhou, cancelada ou validação pendente. |

---

##  Convenções terminológicas

- Porta aberta não significa acesso autorizado. Separar serviço alcançável, autenticação e autorização.
- Capability não é sinônimo de protocolo. SSH e SFTP são meios; capability é a ação possível.
- Execução bem-sucedida não é o mesmo que objetivo validado. Usar validação pós-ação quando aplicável.
- Hash não significa imutabilidade absoluta. Integridade exige controles adicionais.
- Agentless não significa sem dependências. O MVP pode exigir Nmap e OpenSSH.
- Risco pertence à ação e ao contexto, não apenas à ferramenta utilizada.
- Dry-run deve impedir alterações reais no alvo

