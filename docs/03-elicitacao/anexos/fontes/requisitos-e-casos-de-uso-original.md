# NetHub — Documento de Requisitos
## 1. Controle do documento
### 1.1. Histórico de versões

| Versão | Data | Descrição |
|---|---|---|
| 0.1 | 2026-10-13 | Versão inicial do documento |
| 1.0 | 2026-10-15 | Inclusão de versionamento, 
| 1. 1 | 2026-10-15 | Inclusão de versionamento, Escopo


# 2. Escopo

## 2.1. Funcionalidades incluídas

O sistema contempla:

- descoberta de dispositivos utilizando Nmap;
- interpretação de resultados XML;
- manutenção do inventário de hosts;
- consulta de capacidades;
- execução de comandos remotos via SSH;
- consulta de saúde do host;
- gerenciamento de serviços systemd;
- transferência de arquivos por SFTP;
- simulação e execução de workflows;
- registro e consulta de auditoria;
- organização de hosts por tags;
- cadastro de usuários;
- listagem de usuários;
- cadastro, consulta, atualização e remoção de hosts;
- persistência em memória;
- persistência em arquivo binário ou SQLite;
- seleção da persistência na inicialização;
- contagem de entidades cadastradas;
- interface TUI.

## 2.2. Fora do escopo

Não fazem parte da primeira versão:

- gerenciamento de contas do sistema operacional remoto;
- interface web;
- execução de comandos interativos;
- monitoramento contínuo em tempo real;
- sincronização entre múltiplas instâncias;
- execução em redes sem autorização;
- armazenamento de senhas em texto puro;
- gerenciamento de certificados digitais;
- descoberta automática de escopos de rede não autorizados.

---

##3. Requisitos 
#3.1 REQUISITOS FUNCIONAIS 

## RF01 — Descoberta de rede

O sistema DEVE permitir executar uma varredura em escopo autorizado utilizando Nmap, interpretar o arquivo XML gerado e atualizar o inventário de hosts.
O sistema DEVE permitir executar uma varredura em escopo autorizado utilizando Nmap, interpretar o resultado XML e atualizar o inventário de hosts.

**Prioridade:** Essencial  
**Caso de uso:** UC01
-

### Critérios de aceitação

- O escopo deve ser validado antes da execução.
- O Nmap deve ser executado somente no escopo autorizado.
- O resultado XML deve ser interpretado.
- Hosts válidos devem ser inseridos ou atualizados.
- Falhas não devem ser apresentadas como descoberta concluída.


## RF02 — Inventário de hosts

O sistema DEVE armazenar dados estruturados dos hosts, incluindo IP, hostname, portas, serviços, timestamps, capacidades e tags.

**Prioridade:** Essencial  
**Casos de uso:** UC01, UC02, UC09, UC12, UC13

## RF03 — Menu contextual de ações

O sistema DEVE apresentar ações compatíveis com as capacidades confirmadas do host.

**Prioridade:** Essencial  
**Caso de uso:** UC02

## RF04 — Modelo de risco

O sistema DEVE associar níveis de risco às ações e solicitar confirmação proporcional ao risco.

**Prioridade:** Essencial  
**Casos de uso:** UC03, UC05, UC06, UC07

## RF05 — Execução remota

O sistema DEVE executar comandos não interativos via SSH em hosts autorizados, com timeout e registro da tentativa.

**Prioridade:** Essencial  
**Caso de uso:** UC03

## RF06 — Cadastro de usuários

O sistema DEVE permitir cadastrar usuários com login e senha válidos.

**Prioridade:** Essencial  
**Caso de uso:** UC10

## RF07 — Workflow com dry-run

O sistema DEVE permitir simular workflows antes da execução real.

**Prioridade:** Essencial  
**Caso de uso:** UC07

## RF08 — Validação pós-ação

O sistema DEVE verificar se o estado resultante de uma ação corresponde ao estado esperado.

**Prioridade:** Essencial  
**Casos de uso:** UC03, UC05, UC06, UC07

## RF09 — Registro de evidências

O sistema DEVE registrar evidências de ações concluídas, falhas ou canceladas.

**Prioridade:** Essencial  
**Casos de uso:** UC01, UC03, UC04, UC05, UC06, UC07, UC08, UC13

## RF10 — Interface TUI

O sistema DEVE oferecer navegação por terminal com menus, atalhos e feedback visual.

**Prioridade:** Essencial  
**Casos de uso:** UC01 a UC14

## RF11 — Health check

O sistema DEVE consultar disco, memória, carga de CPU e serviços systemd.

**Prioridade:** Importante  
**Caso de uso:** UC04

## RF12 — Gestão de serviços systemd

O sistema DEVE permitir consultar e reiniciar serviços systemd, realizando verificação posterior.

**Prioridade:** Importante  
**Caso de uso:** UC05

## RF13 — Transferência de arquivos

O sistema DEVE permitir upload e download por SFTP e verificar integridade por checksum quando aplicável.

**Prioridade:** Importante  
**Caso de uso:** UC06

## RF14 — Histórico consultável

O sistema DEVE permitir consultar registros por host, ação, resultado e período.

**Prioridade:** Importante  
**Caso de uso:** UC08

## RF15 — Tags de organização

O sistema DEVE permitir criar, associar e consultar tags de hosts.

**Prioridade:** Desejável  
**Caso de uso:** UC09


## RF16 — Listagem de usuários

O sistema DEVE permitir listar todos os usuários sem exibir suas senhas.

**Prioridade:** Essencial  
**Caso de uso:** UC11


## RF17 — Suportar modos de persistência

O sistema DEVE suportar e permitir seleção:

1. armazenamento em memória utilizando uma coleção;
2. armazenamento permanente utilizando arquivo binário ou SQLite.

**Prioridade:** Essencial  
**Aplicação:** Inicialização da aplicação

A seleção da persistência é uma configuração arquitetural e não constitui, obrigatoriamente, um caso de uso independente.

## RF18 — Cadastrar host

O sistema DEVE permitir cadastrar um host associado a um usuário responsável.

**Prioridade:** Essencial  
**Caso de uso:** UC12

## RF19 — Listar hosts

O sistema DEVE permitir listar os hosts cadastrados e aplicar filtros.

**Prioridade:** Essencial  
**Caso de uso:** UC13




---

#3.1 REQUISITOS NÃO-FUNCIONAIS 

# 9. Requisitos não funcionais

## RNF-01 — Usabilidade da interface

A interface de terminal DEVE apresentar menus, comandos e mensagens de forma consistente, permitindo que o operador compreenda o estado da operação e o próximo passo necessário.

### Critérios de aceitação

- Toda operação deve informar seu resultado.
- Mensagens de erro devem indicar o problema e a ação esperada do operador.
- Operações destrutivas ou de risco elevado devem apresentar confirmação explícita.
- O operador deve conseguir retornar ao menu anterior sem encerrar a aplicação.
- Dados indisponíveis devem ser apresentados como indisponíveis.

**Aplicabilidade:** UC01 a UC14.

---

## RNF-02 — Consistência de navegação

A interface DEVE manter o mesmo padrão de navegação em todas as funcionalidades.

### Critérios de aceitação

- Menus devem utilizar nomenclatura uniforme.
- As opções de confirmação e cancelamento devem possuir comportamento consistente.
- O sistema não deve exigir que o operador conheça detalhes internos de arquivos, tabelas ou classes.
- Após uma operação, o sistema deve retornar ao contexto funcional correspondente.

**Aplicabilidade:** UC01 a UC14.

---

## RNF-03 — Proteção de credenciais

O sistema DEVE proteger as credenciais dos usuários e as informações de autenticação utilizadas nas operações remotas.

### Critérios de aceitação

- Senhas não devem ser armazenadas em texto puro.
- Senhas não devem ser exibidas na listagem de usuários.
- Credenciais não devem ser gravadas nos registros de auditoria.
- Mensagens de erro não devem revelar senhas, chaves ou tokens.
- Dados sensíveis não devem ser incluídos em mensagens de depuração destinadas ao usuário.

**Aplicabilidade:** UC03, UC05, UC06, UC10 e UC11.

---

## RNF-04 — Prevenção de execução acidental

O sistema DEVE impedir que operações críticas sejam executadas sem confirmação explícita.

### Critérios de aceitação

- Operações de risco alto ou crítico devem solicitar confirmação.
- A confirmação deve apresentar o host e a ação que será executada.
- O cancelamento da confirmação deve impedir a execução.
- Workflows que contenham ações críticas devem apresentar uma simulação antes da execução.
- A remoção de hosts deve exigir confirmação explícita.

**Aplicabilidade:** UC03, UC05, UC06, UC07 e UC13.

---

## RNF-05 — Controle de autorização

O sistema DEVE verificar se o host e a operação estão autorizados antes de executar qualquer ação remota.

### Critérios de aceitação

- Operações devem ser restritas aos hosts registrados ou explicitamente autorizados.
- Ações incompatíveis com as capacidades conhecidas não devem ser disponibilizadas.
- A ausência de autorização deve bloquear a execução.
- A tentativa de execução não autorizada deve ser registrada.

**Aplicabilidade:** UC01, UC02, UC03, UC05, UC06 e UC07.

---

## RNF-06 — Auditabilidade

O sistema DEVE manter registros suficientes para identificar as operações realizadas e seus resultados.

### Cada registro deve conter, quando aplicável

- identificador da operação;
- data e hora;
- operador;
- host;
- tipo da ação;
- resultado;
- código de saída;
- motivo de falha;
- indicação de cancelamento;
- evidências disponíveis.

### Critérios de aceitação

- Toda operação iniciada deve gerar um registro.
- Falhas e cancelamentos também devem ser registrados.
- O histórico não deve ser alterado por operações de consulta.
- A remoção de um host não deve apagar seus registros históricos.

**Aplicabilidade:** UC01, UC03, UC04, UC05, UC06, UC07, UC08 e UC13.

---

## RNF-07 — Confiabilidade

O sistema DEVE tratar falhas de rede, autenticação, leitura, escrita e execução de processos externos sem perder a consistência dos dados locais.

### Critérios de aceitação

- Falhas de conexão devem ser informadas ao operador.
- Operações interrompidas não devem ser apresentadas como concluídas.
- Falhas de persistência não devem confirmar a operação como bem-sucedida.
- O sistema deve utilizar timeout nas operações remotas.
- A aplicação não deve ser encerrada inesperadamente devido a uma falha operacional tratável.

**Aplicabilidade:** UC01, UC03, UC04, UC05, UC06, UC10, UC11, UC12 e UC13.

---

## RNF-08 — Tratamento de erros

O sistema DEVE tratar erros por meio de exceções e mensagens adequadas ao contexto da operação.

### Critérios de aceitação

- Exceções de validação devem identificar o dado inválido.
- Exceções de persistência devem preservar a causa original do erro.
- Erros técnicos não devem ser apresentados com rastreamentos internos ao operador final.
- O sistema deve diferenciar falha, cancelamento e operação não executada.
- Após um erro recuperável, o sistema deve permitir nova operação.

---

## RNF-09 — Desempenho

O sistema DEVE responder às operações locais dentro de tempo compatível com uma aplicação de terminal.

### Critérios de aceitação

- Consultas em memória devem ser realizadas sem atrasos perceptíveis.
- A listagem de usuários e hosts deve ser realizada em até 2 segundos para o volume previsto na primeira versão.
- A inicialização da aplicação deve ocorrer em até 5 segundos, desconsiderando o tempo de conexão com serviços externos.
- Operações remotas devem respeitar o timeout configurado.
- O tempo de execução de Nmap, SSH e SFTP deve ser apresentado como dependente do ambiente remoto.

---

## RNF-10 — Escalabilidade

A arquitetura DEVE permitir o crescimento da quantidade de usuários, hosts e registros de auditoria sem alteração das regras de negócio.

### Critérios de aceitação

- A aplicação deve utilizar interfaces de repositório.
- O código de domínio não deve depender de uma implementação específica de persistência.
- A substituição do armazenamento em memória por SQLite ou arquivo não deve exigir alterações nos casos de uso.
- Consultas devem utilizar filtros quando o volume de registros crescer.
- O armazenamento de histórico deve permitir consultas por host, ação, resultado e período.

---

## RNF-11 — Manutenibilidade

O sistema DEVE possuir organização modular e responsabilidades bem definidas.

### Critérios de aceitação

- Entidades devem permanecer separadas da interface de terminal.
- Controllers devem coordenar os casos de uso sem conter detalhes de infraestrutura.
- Regras de validação devem ser implementadas em componentes próprios.
- Repositórios devem ser acessados por interfaces.
- Adaptadores de Nmap, SSH, SFTP e SQLite devem permanecer isolados da lógica de negócio.
- Alterações em uma forma de persistência não devem exigir alterações nos demais módulos.

---

## RNF-12 — Testabilidade

Os componentes do sistema DEVEM permitir testes isolados das regras de negócio e dos casos de uso.

### Critérios de aceitação

Devem ser testáveis, no mínimo:

- validação de login;
- validação de senha;
- cadastro de usuário;
- listagem de usuários;
- CRUD de hosts;
- associação entre usuário e host;
- persistência em memória;
- persistência permanente;
- tratamento de falhas de leitura e escrita;
- cálculo do total de entidades;
- confirmação de operações críticas.

---

## RNF-13 — Portabilidade

O sistema DEVE executar em ambientes compatíveis com a plataforma definida para o projeto, desde que as dependências externas estejam disponíveis.

### Critérios de aceitação

- O sistema deve documentar a versão da linguagem utilizada.
- As dependências devem ser declaradas no sistema de build.
- A aplicação deve informar quando Nmap, SSH, SFTP ou SQLite não estiverem disponíveis.
- A ausência de uma ferramenta externa não deve resultar em comportamento silenciosamente incorreto.

---

## RNF-14 — Integridade dos dados

O sistema DEVE preservar a consistência dos dados durante operações de inclusão, atualização, remoção e persistência.

### Critérios de aceitação

- Um usuário inválido não deve ser armazenado.
- Um host associado a usuário inexistente não deve ser armazenado.
- Uma atualização inválida não deve substituir os dados válidos existentes.
- Uma falha de escrita não deve ser apresentada como persistência concluída.
- O identificador do host deve ser preservado em atualizações.
- Registros de auditoria devem ser preservados após a remoção de um host.

---

## RNF-15 — Interoperabilidade com serviços externos

O sistema DEVE utilizar interfaces isoladas para comunicação com Nmap, SSH, SFTP e systemd.

### Critérios de aceitação

- A lógica de negócio não deve executar comandos externos diretamente.
- Cada integração deve possuir um adaptador próprio.
- Falhas dos serviços externos devem ser convertidas em resultados ou exceções tratáveis.
- A substituição de um adaptador não deve exigir alterações na interface de terminal.

---

## RNF-16 — Conformidade e uso autorizado

O sistema DEVE ser utilizado somente em redes, hosts e serviços para os quais o operador possua autorização.

### Critérios de aceitação

- O escopo da descoberta deve ser informado ou configurado.
- A execução de operações remotas deve depender de autorização.
- Operações realizadas devem ser registradas.
- O sistema não deve oferecer ações incompatíveis com as capacidades verificadas do host.

---

# 10. Regras de negócio

## RN01 — Ações críticas

Ações classificadas como de risco alto ou crítico exigem confirmação explícita.

## RN02 — Capacidades desconhecidas

Uma capacidade não verificada não pode ser considerada disponível ou autorizada.

## RN03 — Registro das operações

Toda operação iniciada deve gerar uma evidência de execução, falha ou cancelamento.

## RN04 — Compatibilidade das ações

O sistema deve apresentar somente ações compatíveis com as capacidades confirmadas do host.

## RN05 — Simulação de ações críticas

Workflows que contenham operações críticas devem ser simulados antes da execução.

## RN06 — Proteção de credenciais

Senhas, chaves e tokens não devem ser armazenados em texto puro nem registrados no histórico.

## RN07 — Abordagem agentless

As operações remotas devem utilizar ferramentas e protocolos disponíveis no host, sem exigir a instalação de um agente específico.

## RN08 — Operação local-first

O sistema deve priorizar a consulta dos dados armazenados localmente e utilizar conexões remotas somente quando necessárias.

## RN09 — Associação entre usuário e host

Um host pode ser associado a um usuário responsável pelo cadastro ou gerenciamento.

## RN10 — Identificador do host

O identificador do host deve permanecer inalterado durante uma atualização.

## RN11 — Preservação do histórico

A remoção de um host não deve remover automaticamente os registros de auditoria relacionados.

## RN12 — Seleção única da persistência

O mecanismo de persistência deve ser definido durante a inicialização e utilizado pelos controllers durante a execução da aplicação.

| **RF-01** | Descoberta de Rede | Executar varredura via Nmap, ler o arquivo XML gerado e popular os hosts no inventário SQLite. | Essencial |
-
| **RF-02** | Inventário de Hosts |  Classificar e Armazenar dados estruturados dos hosts: IP,SSH, hostname, portas abertas, serviços, timestamps e tags. | Essencial |
-
| **RF-03** | Menu Contextual de Ações | Exibir dinamicamente ações compatíveis com as capacidades detectadas em cada host. | Essencial |
-
| **RF-04** | Modelo de Risco | Associar níveis de risco (Baixo, Médio, Alto) às ações e exigir confirmação proporcional. | Essencial |
-
| **RF-05** | Execução Remota | Executar comandos não interativos via SSH, com geração obrigatória de auditoria. | Essencial |
-
| **RF-7** | Workflow com Dry-Run | Simular ações antes da execução real, apresentando um relatório prévio do que será feito. | Essencial |
-
| **RF-8** | Validação Pós-Ação | Confirmar e homologar o resultado prático da ação executada no ambiente. | Essencial |
-
| **RF-9** | Registro de Evidências | Gravar logs de forma auditavel | Essencial |
-
| **RF-10** | Interface TUI | Oferecer navegação rica via terminal (Text User Interface), com atalhos e feedback visual rápido. | Essencial |
-
| **RF-11** | Health Check | Verificar métricas de disco, memória, carga de CPU e status de serviços Systemd . | Importante |
-
| **RF-12** | Gestão de Serviços Systemd | Consultar status e reiniciar serviços remotos, com verificação automatizada pós-ação. | Importante |
-
| **RF-13** | Transferência de Arquivos | Realizar upload e download de arquivos via SFTP, com validação de integridade por checksum. | Importante |
-
| **RF-14** | Histórico Consultável | Permitir a filtragem e consulta de evidências armazenadas por host, ação executada e período. | Importante |
-
| **RF-15** | Tags de Organização | Permitir a criação e vinculação manual de tags personalizadas para agrupamento de dispositivos. | Desejável |
---

##Requisitos Não Funcionais
| **RNF-01** | Usabilidade | Interface TUI intuitiva e responsiva 
# 11. Persistência e arquitetura

| **RNF-03** | Segurança | Proteção de credenciais confidenciais
## 11.1 Mecanismos de persistência

| **RNF-04** | Segurança | Prevenção de execuções acidentais críticas 
O sistema deve possuir duas formas de armazenamento:

| **RNF-06** | Auditabilidade | Integridade e rastreabilidade de dados 
### Armazenamento em memória

| **RNF-07** | Confiabilidade | Resiliência a falhas de conexão 
- utiliza uma coleção;
- permite execução rápida e testes;
- perde os dados ao encerrar a aplicação.

| **RNF-08** | Manutenibilidade | Arquitetura de código limpa e modular 
### Armazenamento permanente

| **RNF-09** | Escalabilidade | Suporte a crescimento de escopo atendendo tempo habil 
| **RNF-10** | Conformidade | Validação legal e autorização de uso 
| **RNF-11** | Evolutividade | A arquitetura deve permitir adicionar capacidades e adaptadores sem exigir reescrever toda a aplicação. 
Pode ser implementado por:

##Regras de Negócio
| RN-01 | Ações de risco alto ou crítico exigem confirmação explícita.          
| RN-02 | Capacidade "autorizado" só é atribuída quando sudo está disponível.   
| RN-03 | Toda ação executada gera evidência.                                   
| RN-04 | Ações só são exibidas se compatíveis com as capacidades detectadas.   
| RN-05 | Dry-run é obrigatório para ações de risco crítico.                    
| RN-06 | Credenciais não são persistidas sem consentimento explícito.
| RN-07 | Abordagem Agentless, Local-first
- arquivo binário; ou
- banco de dados SQLite.

A implementação escolhida deve oferecer operações de leitura, inclusão, alteração e remoção para as entidades necessárias. | Credenciais não são persistidas sem consentimento explícito.
| RN-07 | Abordagem Agentless, Local-first

