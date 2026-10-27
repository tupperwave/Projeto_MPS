# Design

## Context

O container já monta `InventoryService`, `UserService` e serviços de operação e auditoria. `Host` é persistido em SQLite e `Usuario` tem repositórios em memória e SQLite. A especificação da Sprint 3 adiciona um registro independente por host e uma fachada de aplicação.

## Goals / Non-Goals

**Goals:**
- Implementar CRUD persistente de anotações e validar a associação com hosts existentes.
- Fornecer acesso singleton aos gerentes e calcular uma contagem consistente dos três tipos de entidade da RF31.
- Preservar a composição por injeção de dependências usada pelos testes.

**Non-Goals:**
- Expor operações de anotação em uma interface CLI/TUI nesta sprint.
- Contar evidências de auditoria como entidades cadastradas na contagem RF31.

## Decisions

- Modelar `AnotacaoHost` com identificador inteiro, IP do host, conteúdo e timestamps de criação/atualização. A anotação não duplica os dados do host; uma chave estrangeira vincula seu ciclo de vida ao registro de host.
- Criar porta de repositório, serviço de aplicação e adaptador SQLite. As operações de leitura retornam `None` ou lista vazia para ausências; atualizar/excluir uma chave inexistente retorna resultado explícito de não encontrado.
- Fazer a camada de aplicação verificar a existência do host antes de criar a anotação. Na persistência, usar `ON DELETE CASCADE` para não deixar registros órfãos.
- Implementar `FacadeSingletonController` como singleton de escopo da composição da aplicação e injetar os gerentes existentes e o serviço de anotações. A composição do container mantém a mesma instância enquanto ela estiver ativa; cada container criado para uma configuração isolada recebe sua própria composição para não compartilhar banco ou serviços entre testes/aplicações.
- Calcular a RF31 somando hosts do inventário, usuários e anotações. Consultar os serviços/repositórios existentes, sem manter contadores duplicados que poderiam divergir após atualizações ou exclusões.
- A migração adiciona a tabela de anotações sem modificar tabelas ou dados atuais.

## Risks / Trade-offs

- A contagem consulta os três tipos de registro em cada chamada → mantém o resultado correto sem sincronização de contadores; o custo é proporcional aos registros até que sejam adicionados métodos de contagem agregada.
- Singleton global pode misturar dependências de aplicações ou testes → limitar sua identidade à composição criada por cada container, conforme RF30.

## Migration Plan

Aplicar a nova migração versionada ao inicializar o banco. Rollback consiste em restaurar o snapshot do banco anterior à migração; os registros da nova tabela serão perdidos ao remover a tabela.
