# Spec Delta

## Purpose

Permite registrar informações complementares associadas a hosts e manter esses registros durante seu ciclo de vida, com acesso consistente aos gerentes do sistema.

## ADDED Requirements

### Requirement: RF29 — Manter anotações de hosts
O sistema DEVE permitir criar, consultar, atualizar e excluir anotações associadas a um host existente.

**Prioridade:** Essencial  
**Casos de uso:** UC15

#### Scenario: Criar anotação para host existente
- **GIVEN** o host está cadastrado no inventário
- **WHEN** o operador cria uma anotação com conteúdo válido para esse host
- **THEN** o sistema persiste a anotação e a disponibiliza na consulta do host

#### Scenario: Rejeitar anotação sem host correspondente
- **GIVEN** não existe host com o identificador informado
- **WHEN** o operador tenta criar ou associar uma anotação
- **THEN** o sistema informa que o host não foi encontrado e não persiste a anotação

#### Scenario: Rejeitar conteúdo vazio
- **GIVEN** o operador envia conteúdo vazio ou composto somente por espaços
- **WHEN** tenta criar ou atualizar uma anotação
- **THEN** o sistema rejeita a operação e mantém os dados existentes

#### Scenario: Consultar, atualizar e excluir anotação
- **GIVEN** existe uma anotação cadastrada
- **WHEN** o operador consulta, atualiza ou exclui a anotação pelo identificador
- **THEN** o sistema retorna os dados atuais, salva a atualização ou remove o registro conforme a operação solicitada

### Requirement: RF30 — Acessar gerentes por fachada única
O sistema DEVE oferecer um ponto único que disponibilize os gerentes de aplicação necessários aos fluxos do sistema.

**Prioridade:** Essencial  
**Casos de uso:** UC15

#### Scenario: Obter fachada dos gerentes
- **GIVEN** a aplicação foi inicializada
- **WHEN** um cliente solicita a fachada dos gerentes
- **THEN** recebe a instância única configurada para aquela aplicação e seus gerentes de inventário, usuários e anotações

### Requirement: RF31 — Contar entidades cadastradas
O sistema DEVE informar a quantidade total de hosts, usuários e anotações atualmente cadastrados.

**Prioridade:** Essencial  
**Casos de uso:** UC15

#### Scenario: Consultar total de entidades
- **GIVEN** existem hosts, usuários e anotações persistidos no sistema
- **WHEN** um cliente solicita a quantidade de entidades cadastradas
- **THEN** o sistema retorna a soma dos registros desses três tipos, sem contar duas vezes uma anotação vinculada a um host

#### Scenario: Nenhuma entidade cadastrada
- **GIVEN** não existem hosts, usuários ou anotações cadastrados
- **WHEN** um cliente solicita a quantidade de entidades cadastradas
- **THEN** o sistema retorna zero
