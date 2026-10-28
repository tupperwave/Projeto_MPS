# Design

## Context

As Sprints 1 e 2 estabeleceram a separação entre domínio, serviços de aplicação, portas e adaptadores de infraestrutura: `UserService` e `InventoryService` recebem contratos, e `bootstrap.py` faz a composição. A Sprint 3 mantém esse limite; os requisitos observáveis continuam definidos em `specs/anotacoes-de-host/spec.md`.

## Goals / Non-Goals

**Goals:**
- Implementar CRUD persistente de anotações e validar a associação com hosts existentes.
- Fornecer uma fachada singleton por composição e calcular a contagem dos tipos da RF31 sem concentrar a regra na fachada.
- Preservar a composição por injeção de dependências usada pelos testes.
- Permitir adicionar novas fontes de contagem sem alterar o serviço de contagem ou a fachada.

**Non-Goals:**
- Expor operações de anotação em uma interface CLI/TUI nesta sprint.
- Contar evidências de auditoria como entidades cadastradas na contagem RF31.

## Decisions

- Modelar `AnotacaoHost` com identificador inteiro, IP do host, conteúdo e timestamps de criação/atualização. A anotação não duplica os dados do host; uma chave estrangeira vincula seu ciclo de vida ao registro de host.
- Criar porta de repositório, serviço de aplicação e adaptador SQLite. As operações de leitura retornam `None` ou lista vazia para ausências; atualizar/excluir uma chave inexistente retorna resultado explícito de não encontrado.
- Fazer a camada de aplicação verificar a existência do host antes de criar a anotação. Na persistência, usar `ON DELETE CASCADE` para não deixar registros órfãos.
- Agrupar os gerentes nomeados em `GerentesController`; a fachada recebe o grupo e não replica oito parâmetros individuais nem contém a lógica de contagem.
- Implementar `FacadeSingletonController` como singleton por `FacadeSingletonScope`, usando uma chave fraca e sincronizada. O escopo pertence ao container, e a fachada não mantém referência ao escopo; composições isoladas não compartilham dependências e a cache não retém composições encerradas.
- Extrair a contagem para `EntityCountService`, que soma provedores estruturais de contagem. A fachada mantém o método exigido pela RF31 e apenas delega ao serviço.
- Definir `EntityCountSource` separado das portas CRUD. Cada repositório concreto oferece contagem eficiente (`COUNT(*)` no SQLite ou `len` em memória) sem obrigar todos os clientes CRUD a dependerem de uma operação de contagem.
- Injetar a coleção de fontes na composição do container. A inclusão de um novo tipo contado adiciona seu provedor à coleção, sem alterar o algoritmo de `EntityCountService` nem a fachada.
- Tratar o container como dono da conexão SQLite: oferece fechamento explícito e a CLI fecha somente containers que ela própria criou. Um finalizador serve como proteção para composições descartadas sem fechamento explícito.
- A migração adiciona a tabela de anotações sem modificar tabelas ou dados atuais.

## Risks / Trade-offs

- O total inclui somente as categorias definidas pela RF31 → novas categorias exigem atualização da composição e da especificação quando a regra observável mudar.
- A coleção de fontes pode omitir uma entidade por erro de composição → manter teste de total combinado que falha quando uma categoria prevista deixar de ser contada.

## Migration Plan

Aplicar a nova migração versionada ao inicializar o banco. Rollback consiste em restaurar o snapshot do banco anterior à migração; os registros da nova tabela serão perdidos ao remover a tabela.
