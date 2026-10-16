# Gestão de usuários e hosts

## Purpose

Definir o cadastro e a consulta de usuários e hosts, incluindo validações, persistência e resumo de entidades.

## Requirements

### Requirement: RF16 — Cadastrar usuário
O sistema DEVE permitir cadastrar um novo usuário com login único.

**Prioridade:** Essencial
**Casos de uso:** UC10

#### Scenario: Cadastro de usuário válido
- **GIVEN** o operador informou login e senha válidos
- **WHEN** confirma o cadastro
- **THEN** o sistema persiste o usuário e informa sucesso

### Requirement: RF17 — Listar usuários
O sistema DEVE permitir consultar os usuários cadastrados sem expor credenciais.

**Prioridade:** Essencial
**Casos de uso:** UC11

#### Scenario: Listagem sem credenciais
- **GIVEN** existem usuários cadastrados
- **WHEN** o operador solicita a listagem
- **THEN** o sistema apresenta os usuários sem exibir senhas ou dados sensíveis

### Requirement: RF18 — Proteger senha no armazenamento
O sistema DEVE proteger a senha antes de persistir os dados de usuário.

**Prioridade:** Essencial
**Casos de uso:** UC10

#### Scenario: Persistir senha protegida
- **GIVEN** o operador forneceu uma senha válida
- **WHEN** o sistema realiza o cadastro
- **THEN** a senha é armazenada em formato protegido e não em texto puro

### Requirement: RF20 — Persistir dados de usuários e hosts
O sistema DEVE persistir os dados de usuários e hosts no mecanismo selecionado pela aplicação.

**Prioridade:** Essencial
**Casos de uso:** UC10, UC11, UC12, UC13

#### Scenario: Salvar dados após operação válida
- **GIVEN** uma operação de cadastro ou atualização foi validada
- **WHEN** o operador confirma a operação
- **THEN** o sistema grava os dados no mecanismo de persistência ativo

### Requirement: RF21 — Tratar falhas de leitura e escrita
O sistema DEVE informar falhas de leitura e escrita sem encerrar abruptamente a aplicação.

**Prioridade:** Essencial
**Casos de uso:** UC10, UC11, UC12, UC13

#### Scenario: Erro de persistência
- **GIVEN** ocorreu falha de leitura ou escrita durante uma operação
- **WHEN** o sistema detecta o erro
- **THEN** o sistema informa a falha e mantém a aplicação disponível

### Requirement: RF22 — Cadastrar host associado a usuário
O sistema DEVE permitir cadastrar host e associá-lo a um usuário responsável existente.

**Prioridade:** Essencial
**Casos de uso:** UC12

#### Scenario: Cadastro de host com responsável existente
- **GIVEN** o usuário responsável já está cadastrado
- **WHEN** o operador informa os dados válidos do host
- **THEN** o sistema registra o host e mantém a associação com o usuário

### Requirement: RF23 — Listar hosts cadastrados
O sistema DEVE listar os hosts cadastrados e permitir a consulta de seus detalhes.

**Prioridade:** Essencial
**Casos de uso:** UC13

#### Scenario: Consultar host da lista
- **GIVEN** existem hosts cadastrados
- **WHEN** o operador seleciona um host na listagem
- **THEN** o sistema apresenta os detalhes do host selecionado

### Requirement: RF24 — Filtrar hosts
O sistema DEVE permitir filtrar hosts por IP, hostname, usuário responsável ou tag.

**Prioridade:** Importante
**Casos de uso:** UC13

#### Scenario: Aplicar filtro por hostname
- **GIVEN** o inventário possui múltiplos hosts
- **WHEN** o operador aplica um filtro de hostname
- **THEN** o sistema apresenta apenas os hosts correspondentes ao filtro

### Requirement: RF25 — Atualizar host
O sistema DEVE permitir atualizar os dados de um host preservando seu identificador.

**Prioridade:** Importante
**Casos de uso:** UC13

#### Scenario: Atualização de host existente
- **GIVEN** um host existente foi selecionado
- **WHEN** o operador informa dados válidos para atualização
- **THEN** o sistema salva os novos dados mantendo o identificador original

### Requirement: RF26 — Remover host com confirmação
O sistema DEVE exigir confirmação explícita antes de remover host do inventário.

**Prioridade:** Importante
**Casos de uso:** UC13

#### Scenario: Confirmar remoção de host
- **GIVEN** um host existente foi selecionado para remoção
- **WHEN** o operador confirma explicitamente a operação
- **THEN** o sistema remove o host e registra o evento de remoção na auditoria

### Requirement: RF27 — Consultar total de usuários
O sistema DEVE informar a quantidade total de usuários cadastrados.

**Prioridade:** Desejável
**Casos de uso:** UC17

#### Scenario: Exibir total de usuários
- **GIVEN** a fachada da aplicação está inicializada
- **WHEN** o operador solicita o resumo
- **THEN** o sistema apresenta o total de usuários cadastrados

### Requirement: RF28 — Consultar total de hosts e total geral
O sistema DEVE informar a quantidade total de hosts e o total geral de entidades.

**Prioridade:** Desejável
**Casos de uso:** UC17

#### Scenario: Exibir totais consolidados
- **GIVEN** a fachada da aplicação está inicializada
- **WHEN** o operador solicita o resumo de entidades
- **THEN** o sistema apresenta o total de hosts e o total geral consolidado
