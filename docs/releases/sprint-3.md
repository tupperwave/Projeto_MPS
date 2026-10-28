# Release da Sprint 3 — Padrões 1

**Data:** 27/10/2026  
**Tag:** `sprint-3`  
**Base:** `entrega-1` (Sprints 1 e 2)

## Entregas

- CRUD persistente de anotações de hosts, com vínculo ao inventário.
- `FacadeSingletonController` com os gerentes da aplicação e contagem de hosts, usuários e anotações.
- Comandos de CLI para criar, obter, listar, atualizar e excluir anotações.
- Diagrama de classes com fronteira, controle, modelo, fachada, persistência e detector de rede local.
- Detecção automática do CIDR local para a descoberta; a CLI mostra o escopo e pede autorização.
- README e UC15 atualizados.

## Validação

- Suíte completa: `python3 -m unittest discover -v` — **65 testes aprovados**.
- Especificações: `openspec validate --all` — **13 itens válidos**, com avisos recomendando palavras-chave RFC em inglês nos requisitos escritos em português.

## Notas por commit desde `entrega-1`

| Commit | Assunto | Comentário |
|---|---|---|
| `ce7a203` | Atualiza checklist da Entrega 1 | Registra o andamento e os itens concluídos das Sprints 1 e 2. |
| `c070e0e` | Resolve conflito da documentação | Consolida as mudanças concorrentes no checklist da Entrega 1. |
| `7dde047` | Merge do PR #14 | Integra a documentação da Entrega 1 à `main`. |
| `c5669b2` | Specs e CRUD de anotações | Especifica RF29–RF31 e implementa entidade, serviço e persistência SQLite para anotações vinculadas a hosts. |
| `6ea6c15` | Adiciona UC15 | Documenta os fluxos de manutenção de anotações e consulta da fachada. |
| `9f85f8f` | Registra UC15 na branch de documentação | Reaplica a documentação do caso de uso que foi integrada pelo PR #15. |
| `63f8c97` | Merge do PR #15 | Integra UC15 à `main`. |
| `d8882b0` | Upload inicial da fachada | Adiciona a primeira versão de `FacadeSingletonController`, com Singleton por composição e contagem de entidades. |
| `ad0f2b2` | Compõe a fachada no bootstrap | Monta a fachada com os gerentes existentes e a expõe no container. |
| `ea02da1` | Testes da fachada | Adiciona testes para Singleton, dependências, gerentes expostos e contagem. |
| `9badb34` | Integra fachada e testes | Move a fachada para a camada de aplicação, integra o bootstrap e inclui testes de CRUD e contagem. |
| `cc871ba` | Atualiza UC15 e diagrama | Reflete os fluxos RF29–RF31 e as classes de anotação, persistência e fachada. |
| `a52908e` | Refatora para SOLID | Separa o serviço de contagem, agrupa os gerentes e limita o ciclo de vida do Singleton à composição. |
| `5689c56` | Merge da Sprint 3 | Integra a implementação inicial da Sprint 3 à `main`. |
| `1f2aedf` | Atualiza checklist do release | Registra a integração da Sprint 3 e a tag de entrega. |
| `253eb3f` | Documenta anotações no README | Acrescenta instruções de acesso ao CRUD da camada de aplicação. |
| `ec02804` | Merge da atualização do README | Integra as instruções de anotações à `main`. |
| `6e1cd9d` | Detecta rede local automaticamente | Adiciona a detecção do CIDR da rota padrão, mantém a confirmação e documenta a instalação do Nmap. |
| `0222387` | Merge da detecção de rede | Integra a descoberta automática à `main`. |
| `3303460` | Expõe CRUD de anotações na CLI | Adiciona os subcomandos de anotações, atualiza UC15 e documenta os estereótipos BCE no diagrama. |
| `65ef5a9` | Merge do CRUD e diagrama | Integra a versão atualizada da Sprint 3 à `main`. |
