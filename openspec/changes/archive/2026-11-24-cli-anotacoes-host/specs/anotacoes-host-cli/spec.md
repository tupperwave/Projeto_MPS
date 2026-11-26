## ADDED Requirements

### Requirement: RF32 — Gerenciar anotações de hosts pela CLI
O sistema DEVE disponibilizar na CLI operações para criar, consultar por identificador, listar (com filtro opcional por host), atualizar e excluir anotações de hosts, delegando as operações ao serviço de anotações pela fachada dos gerentes.

**Prioridade:** Essencial
**Casos de uso:** UC15

#### Scenario: Executar CRUD de anotações pela CLI
- **GIVEN** existe um host no inventário e a aplicação foi iniciada
- **WHEN** o operador cria, consulta, lista, atualiza ou exclui uma anotação pela CLI
- **THEN** a CLI apresenta o resultado e mantém as alterações na persistência SQLite

#### Scenario: Consultar anotações sem resultados
- **GIVEN** não há anotações para o filtro informado
- **WHEN** o operador lista anotações pela CLI
- **THEN** o sistema informa que nenhuma anotação foi encontrada

#### Scenario: Operar com identificador inexistente
- **GIVEN** o identificador não corresponde a uma anotação
- **WHEN** o operador consulta, atualiza ou exclui pela CLI
- **THEN** o sistema apresenta uma mensagem compreensível de registro inexistente
