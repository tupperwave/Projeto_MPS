# Design

## Context

`HostAnnotationService` e a persistência SQLite já implementam o CRUD de anotações. A CLI deve atuar como fronteira e reutilizar a fachada da aplicação para obter o gerente.

## Decisions

- Adicionar `anotacoes` com subcomandos `criar`, `obter`, `listar`, `atualizar` e `excluir`.
- Criar a classe de apresentação `CliAnotacoes` para formatar resultados e delegar operações à `FacadeSingletonController`.
- Usar o banco SQLite padrão da aplicação; exigir que o host exista para criação e aceitar conteúdo entre aspas quando tiver espaços.
- Retornar erro tratado quando o identificador não existir, inclusive na exclusão.
- Marcar `CliAnotacoes` como `<<boundary>>`, os serviços de aplicação como `<<control>>` e as entidades de domínio como `<<model>>` no diagrama da Sprint 3.

## Risks

- Anotações persistem entre execuções pelo banco SQLite padrão; o README deve mostrar como especificar o banco e explicar que o host precisa existir no inventário.
