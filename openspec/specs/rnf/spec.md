# Requisitos não funcionais — critérios mensuráveis

## Purpose

Definir critérios de aceitação mensuráveis para os requisitos não funcionais do NetHub.

## Requirements

### Requirement: NF-US-01 (orig. RNF-01) — Usabilidade
O sistema DEVE permitir que operadores concluam tarefas principais da TUI com eficiência e com baixa taxa de erro.

**Prioridade:** Importante
**Casos de uso:** UC01, UC03, UC08, UC10, UC12, UC13

#### Scenario: Tempo de execução das tarefas principais
- **GIVEN** uma amostra de pelo menos 10 operadores executando tarefas UC01, UC03 e UC08 em ambiente homologado
- **WHEN** o teste de usabilidade for concluído
- **THEN** pelo menos 90% das execuções devem ser concluídas sem ajuda em até 3 minutos por tarefa

#### Scenario: Responsividade da navegação
- **GIVEN** o operador navega entre menus e confirma ações na TUI
- **WHEN** o sistema processa entradas de teclado sem operação remota bloqueante em andamento
- **THEN** o tempo de resposta visual deve ser de até 200 ms no percentil 95

### Requirement: NF-SE-01 (orig. RNF-03) — Segurança de credenciais

O sistema DEVE proteger credenciais confidenciais em armazenamento, exibição e observabilidade.

**Prioridade:** Essencial
**Casos de uso:** UC03, UC05, UC06, UC10, UC11, UC12, UC13

#### Scenario: Credenciais não expostas
- **GIVEN** um conjunto de operações que envolve autenticação e execução remota
- **WHEN** forem inspecionados banco de dados, saídas de tela e logs de execução
- **THEN** deve haver 0 ocorrências de senhas em texto puro

### Requirement: NF-SE-02 (orig. RNF-04) — Prevenção de execução acidental

O sistema DEVE prevenir execuções acidentais de ações classificadas como risco alto.

**Prioridade:** Essencial
**Casos de uso:** UC03, UC05, UC06, UC07, UC13

#### Scenario: Bloqueio sem confirmação explícita
- **GIVEN** uma ação de risco alto foi selecionada
- **WHEN** o operador não confirma explicitamente a execução
- **THEN** a execução não deve ocorrer em 100% dos casos testados

### Requirement: NF-AU-01 (orig. RNF-06) — Auditabilidade

O sistema DEVE garantir rastreabilidade de ações por meio de evidências auditáveis completas.

**Prioridade:** Essencial
**Casos de uso:** UC03, UC04, UC05, UC06, UC07, UC08, UC13

#### Scenario: Cobertura de auditoria
- **GIVEN** operações remotas, consultas críticas e remoções de host foram executadas
- **WHEN** o histórico for consultado
- **THEN** 100% das operações devem possuir registro com host alvo, ação, timestamp e resultado

### Requirement: NF-CO-01 (orig. RNF-07) — Confiabilidade

O sistema DEVE ser resiliente a falhas transitórias de conexão em operações remotas.

**Prioridade:** Importante
**Casos de uso:** UC03, UC04, UC05, UC06, UC07

#### Scenario: Retentativa controlada
- **GIVEN** uma falha transitória de conexão durante operação remota
- **WHEN** o sistema tenta recuperar a execução
- **THEN** devem ser realizadas até 3 tentativas automáticas com timeout máximo de 10 segundos por tentativa antes de reportar falha

### Requirement: NF-MA-01 (orig. RNF-08) — Manutenibilidade

O sistema DEVE manter separação modular para reduzir impacto de mudanças funcionais.

**Prioridade:** Importante
**Casos de uso:** UC01 a UC13, UC17

#### Scenario: Evolução com impacto controlado
- **GIVEN** a implementação de um novo caso de uso funcional
- **WHEN** a alteração for concluída e revisada
- **THEN** a mudança deve ficar restrita aos módulos diretamente relacionados e não deve exigir reescrita global da aplicação

### Requirement: NF-DE-01 (orig. RNF-09) — Desempenho e escalabilidade

O sistema DEVE manter desempenho aceitável para operação em inventários de médio porte.

**Prioridade:** Importante
**Casos de uso:** UC01, UC02, UC08, UC13, UC17

#### Scenario: Consulta de inventário em escala
- **GIVEN** um inventário com 500 hosts cadastrados
- **WHEN** o operador listar hosts com ou sem filtro simples (IP, hostname ou tag)
- **THEN** o tempo de resposta da consulta deve ser de até 2 segundos no percentil 95

### Requirement: NF-CF-01 (orig. RNF-10) — Conformidade

O sistema DEVE executar operações administrativas somente quando houver autorização explícita de uso.

**Prioridade:** Essencial
**Casos de uso:** UC03, UC05, UC06, UC07, UC10, UC12, UC13

#### Scenario: Bloqueio sem autorização registrada
- **GIVEN** um operador sem autorização explícita tenta executar uma ação administrativa
- **WHEN** a solicitação é submetida
- **THEN** a operação deve ser negada em 100% dos casos e o evento deve ser registrado

### Requirement: NF-EV-01 (orig. RNF-11) — Evolutividade

O sistema DEVE permitir adicionar novas capacidades por meio de módulos/adaptadores sem reescrita completa.

**Prioridade:** Importante
**Casos de uso:** UC01 a UC13, UC17

#### Scenario: Inclusão de novo adaptador
- **GIVEN** a equipe precisa integrar uma nova capacidade de operação remota
- **WHEN** um novo adaptador é implementado
- **THEN** a integração deve ocorrer sem reescrever os módulos de domínio já existentes
