# Gerenciamento de usuários

## ADDED Requirements

### Requirement: RF06 — Cadastrar usuários
O sistema DEVE permitir cadastrar usuários com login e senha válidos, usando o repositório associado ao modo de persistência selecionado na inicialização.

**Prioridade:** Essencial  
**Casos de uso:** UC10

#### Scenario: Cadastro com dados válidos
- **GIVEN** a aplicação iniciou com um modo de persistência selecionado
- **AND** o login e a senha atendem às regras de validação
- **WHEN** o operador solicita o cadastro
- **THEN** o usuário é salvo pelo repositório correspondente ao modo selecionado
- **AND** o sistema informa sucesso somente após a gravação ser concluída

#### Scenario: Login inválido
- **GIVEN** o login está vazio, tem mais de 12 caracteres ou contém números
- **WHEN** o operador solicita o cadastro
- **THEN** o sistema rejeita o usuário e informa a regra de login violada

#### Scenario: Senha inválida
- **GIVEN** a senha tem menos de 8 ou mais de 128 caracteres, contém menos de três destes grupos — maiúsculas, minúsculas, números e caracteres não alfanuméricos — ou é igual ao login
- **WHEN** o operador solicita o cadastro
- **THEN** o sistema rejeita o usuário e informa a regra de senha violada

**Regra da sprint:** a senha deve ser diferente do login. Essa exigência vem do enunciado da Sprint 2; não é apresentada como requisito da política padrão do AWS IAM.

#### Scenario: Falha ao salvar usuário
- **GIVEN** o repositório selecionado falha ao gravar o usuário
- **WHEN** o operador solicita o cadastro
- **THEN** o sistema informa a falha e não confirma o cadastro como concluído

### Requirement: RF16 — Listar usuários
O sistema DEVE permitir listar todos os usuários cadastrados sem exibir senhas nem dados de autenticação sensíveis.

**Prioridade:** Essencial  
**Casos de uso:** UC11

#### Scenario: Consultar usuários cadastrados
- **GIVEN** existem usuários no repositório selecionado
- **WHEN** o operador solicita a listagem
- **THEN** o sistema apresenta todos os usuários cadastrados
- **AND** não apresenta senhas nem dados de autenticação sensíveis

#### Scenario: Listagem vazia
- **GIVEN** não há usuários cadastrados
- **WHEN** o operador solicita a listagem
- **THEN** o sistema informa que a coleção está vazia

#### Scenario: Falha ao ler usuários
- **GIVEN** o repositório selecionado falha ao ler os usuários
- **WHEN** o operador solicita a listagem
- **THEN** o sistema informa a falha e não apresenta o resultado como uma listagem concluída

### Requirement: RF17 — Selecionar modo de persistência
O sistema DEVE oferecer armazenamento de usuários em uma coleção mantida em memória e em um mecanismo permanente (arquivo binário ou SQLite), permitindo selecionar um modo na inicialização.

**Prioridade:** Essencial  
**Aplicação:** Inicialização da aplicação  
**Casos de uso:** UC10, UC11

#### Scenario: Selecionar modo em memória
- **GIVEN** a aplicação está iniciando
- **WHEN** o operador seleciona armazenamento em memória
- **THEN** os cadastros e listagens da execução usam uma coleção mantida em RAM

#### Scenario: Selecionar modo permanente
- **GIVEN** a aplicação está iniciando
- **WHEN** o operador seleciona SQLite ou arquivo binário
- **THEN** os cadastros e listagens usam o repositório permanente selecionado

#### Scenario: Falha de entrada ou saída
- **GIVEN** uma operação do repositório permanente falha por erro de entrada ou saída
- **WHEN** o sistema lê ou grava usuários
- **THEN** a falha é tratada e comunicada sem indicar sucesso da operação
