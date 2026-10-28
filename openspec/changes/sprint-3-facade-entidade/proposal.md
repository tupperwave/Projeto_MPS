# Proposal

## Why

A Sprint 3 pede o CRUD de uma nova entidade relacionada ao inventário e um ponto único de acesso para os gerentes da aplicação. Hoje os serviços são montados no container, mas não há uma entidade associada a hosts com ciclo completo de manutenção nem uma fachada com contagem agregada.

## What Changes

- Criar a entidade Anotação de Host, vinculada a um host cadastrado, com cadastro, consulta, atualização e exclusão.
- Persistir anotações no SQLite e disponibilizar seus casos de uso pela aplicação.
- Criar a fachada única `FacadeSingletonController` para acesso aos gerentes, incluindo a quantidade total de hosts, usuários e anotações cadastrados.
- Atualizar o diagrama de classes e o histórico/checklist da Sprint 3.

## Capabilities

### New Capabilities
- `anotacoes-de-host`: CRUD de anotações vinculadas a hosts e acesso unificado aos gerentes, com contagem de entidades cadastradas.

### Modified Capabilities

## Impact

- `nethub/domain/`, `nethub/application/`, `nethub/infrastructure/` e `nethub/bootstrap.py`.
- Migração SQLite e testes automatizados.
- `docs/05-analise.md`, documentação dos casos de uso e `docs/00-entregas.md`.
