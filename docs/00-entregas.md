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

- [ ] Abrir/atualizar o PR de `dev` para `main` e concluir o merge.
- [ ] Criar e publicar a tag `entrega-1` sobre o commit integrado à `main`.

## Sprint 3 — anotações de hosts e fachada de gerentes

- [x] Analisar specs e criar a change OpenSpec `sprint-3-facade-entidade` com requisitos RF29–RF31.
- [x] Implementar a entidade `AnotacaoHost`, CRUD no serviço e persistência SQLite vinculada ao host.
- [x] Criar `FacadeSingletonController` para acesso aos gerentes da aplicação e contagem de hosts, usuários e anotações.
- [x] Criar UC15 e atualizar o diagrama de classes de análise para contemplar a Sprint 3.
- [x] Revisar e validar a implementação e os artefatos da Sprint 3 (61 testes aprovados; specs validadas).
- [x] Separar a sprint em três pacotes ZIP: specs e CRUD; fachada; UC15 e diagrama.
- [ ] Enviar os commits 2 e 3 para `origin/dev` após autorização explícita e concluir a integração no Git.

## Pendências gerais do repositório

- [ ] Confirmar escopo e nomenclatura final do produto.
- [ ] Revisar cenários e critérios mensuráveis dos demais requisitos não funcionais.
- [ ] Completar o diagrama geral de casos de uso e os diagramas de interface e arquitetura.
- [ ] Completar os modelos gerais de classes de análise e projeto e registrar decisões de arquitetura.
