# Entregas

Use esta página como checklist de acompanhamento. As datas e os critérios específicos da turma devem ser confirmados pela equipe.

## Entrega 1 — Sprints 1 e 2

### Documentação e análise

- [x] Organizar visão, glossário e elicitação; registrar que não houve elicitação formal para esta linha de base.
- [x] Especificar RF06, RF16, RF17 e NF-SE-03 com cenários no OpenSpec.
- [x] Criar diagrama UML de casos de uso de usuários com Administrador e Operador autorizado ([PlantUML](04-casos-de-uso/diagrama-usuarios.puml)).
- [x] Atualizar o diagrama de classes de análise com fronteira, controle, entidade, exceções e persistências RAM/SQLite ([análise](05-analise.md#modelo-de-classes--gerenciamento-de-usuários)).

### Sprint 1 — cadastro, coleção em memória e listagem

- [x] Separar o código em domínio, aplicação, infraestrutura e apresentação.
- [x] Implementar cadastro de usuários e armazenamento em coleção RAM.
- [x] Implementar listagem de todos os usuários.

### Sprint 2 — validação e persistência

- [x] Validar login não vazio, com até 12 caracteres e sem números.
- [x] Validar senha de 8 a 128 caracteres, com pelo menos três dos quatro grupos e diferente do login.
- [x] Implementar persistência permanente em SQLite e seleção entre RAM e SQLite na inicialização.
- [x] Tratar erros de persistência, inclusive falhas ao abrir e migrar o banco SQLite.
- [x] Integrar cadastro e listagem à CLI/TUI sem exibir senha, salt ou hash.
- [x] Cobrir os fluxos com testes: `python3 -m unittest discover -v` (53 testes aprovados).
- [x] Validar as especificações OpenSpec: `openspec validate --all` (11 itens válidos; há avisos de recomendação sobre palavras-chave em inglês).

### Integração da entrega

- [x] Integrar `dev` à `main` pelo PR #14.
- [x] Criar e publicar a tag `entrega-1` sobre o commit integrado à `main`.

## Sprint 3 — anotações de hosts e fachada de gerentes

- [x] Analisar specs e criar a change OpenSpec `sprint-3-facade-entidade` com requisitos RF29–RF31.
- [x] Implementar a entidade `AnotacaoHost`, CRUD no serviço e persistência SQLite vinculada ao host.
- [x] Expor criar, consultar, listar, atualizar e excluir anotações pela CLI, delegando à fachada única.
- [x] Criar `FacadeSingletonController` para acesso aos gerentes da aplicação e contagem de hosts, usuários e anotações.
- [x] Criar UC15 e atualizar o diagrama de classes de análise para contemplar a Sprint 3.
- [x] Revisar arquitetura e artefatos da Sprint 3 conforme SOLID, preservando as portas das Sprints 1 e 2 (65 testes aprovados; specs validadas).
- [x] Separar a sprint em três pacotes ZIP: specs e CRUD; fachada; UC15 e diagrama.
- [x] Enviar os commits 2 e 3 para `origin/dev` e integrar a Sprint 3 à `main`.

## Sprint 4 — persistência de acessos e relatórios

- [x] Persistir tentativas de autenticação e produzir estatísticas por usuário.
- [x] Gerar relatórios pelo Template Method nos formatos texto, HTML e PDF; a CLI exporta HTML ou PDF.
- [x] Adaptar o logging da biblioteca padrão à porta `LoggerPort`.
- **Parcial:** `RepositoryFactory` centraliza a composição, mas só os repositórios de usuários alternam entre memória e SQLite; os demais permanecem em SQLite.
- [x] Especificar RF33 e documentar UC16, os formatos e a exportação pela CLI.
- [x] Integrar as mudanças da Sprint 4 à `main` pelos PRs #20, #21 e #22.
- [ ] Criar tag da Sprint 4; não há tag correspondente no repositório neste momento.

## Sprint 5 — padrões Command e Memento

- **Estado:** Tarefa 3.2 integrada em `main` pelo PR #24. `dev` recebeu o PR #23 com uma implementação concorrente cujo Memento fica somente em memória; esta integração adota a versão persistida em SQLite.
- [x] Atualizar a cópia local a partir da `main`, preservando as alterações que já estavam no diretório.
- [x] Encaminhar as operações da fachada para comandos concretos executados por um invoker.
- [x] Implementar Memento para desfazer uma única vez a atualização mais recente de anotação.
- [x] Expor o desfazer pela CLI e manter o snapshot no SQLite entre execuções.
- [x] Atualizar o diagrama de classes, a especificação de anotações e os documentos de projeto.
- [x] Adicionar testes focados em Command, Memento e persistência do último snapshot.
- [x] Executar a suíte completa após a integração: `python3 -m unittest discover -v` (81 testes aprovados).
- [ ] Integrar esta versão reconciliada à branch `dev` por Pull Request, após validar a suíte completa.

## Pendências gerais do repositório

- [x] Consolidar a Versão 2.1 do documento do projeto, com revisões, casos de uso, diagramas e avaliação de implementação até a Sprint 5 ([documento](09-documento-projeto-v2.md)).
- [ ] Reconciliar RF20–RF28 citados em UC12–UC14 e decidir se o requisito de CRUD integral se aplica também a `Usuario`.

- [ ] Confirmar escopo e nomenclatura final do produto.
- [ ] Revisar cenários e critérios mensuráveis dos demais requisitos não funcionais.
- [x] Criar diagrama geral de casos de uso cobrindo os requisitos funcionais definidos ([versão 2](04-casos-de-uso/diagrama-casos-de-uso-v2.puml)).
- [ ] Completar os diagramas de interface e arquitetura.
- [ ] Completar os modelos gerais de classes de análise e projeto e registrar decisões de arquitetura.
