# Design

## Context

A aplicação separa domínio, aplicação e infraestrutura. O domínio contém entidades e exceções; contratos de repositório ficam em `nethub/domain/ports.py`; serviços de aplicação recebem esses contratos; `nethub/bootstrap.py` monta o container. Não há entidade nem repositório de usuários. Ver `specs/gestao-de-usuarios/spec.md` para regras e cenários.

## Goals / Non-Goals

**Goals:**
- Aplicar validação e proteção de senha antes de criar uma entidade de usuário.
- Implementar cadastro e listagem usando uma coleção em memória, injetada na aplicação.
- Evitar expor a senha original pela entidade, sua representação ou pela listagem.

**Non-Goals:**
- Persistência SQLite/arquivo e seleção de backend na inicialização.
- Comandos CLI/TUI e autenticação interativa.
- Atualização e remoção de usuários.

## Decisions

- Criar `Usuario` em `nethub/domain/users.py`, separado do modelo de hosts, com uma fábrica que valida login e senha e guarda somente um hash com salt.
- Usar `hashlib.scrypt` com salt aleatório por usuário. É suportado pela biblioteca padrão e evita uma dependência adicional; o repositório guarda o hash codificado, nunca a senha original. A função de verificação usa comparação em tempo constante.
- Interpretar login vazio após remover espaços nas extremidades; validar limite de 12 caracteres e ausência de dígitos. Comparar logins por igualdade exata, pois a especificação não define normalização de caixa.
- Acrescentar erros específicos de domínio para login/senha inválidos e login duplicado. A camada de aplicação traduz as regras de cadastro para esses erros sem incluir a senha nas mensagens.
- Declarar `UserRepository` em `domain/ports.py`, implementar `MemoryUserRepository` em `infrastructure/`, e fornecer `UserService` em `application/` para cadastrar e listar. O container injeta o repositório em memória como modo atual; seleção do backend fica para etapa futura.

## Risks / Trade-offs

- Scrypt usa mais memória e CPU que um hash rápido -> configurar seus parâmetros em constantes nomeadas e gerar o hash apenas no cadastro; validar o tempo em equipamento-alvo quando a autenticação for integrada.
- Comparação de login sensível a maiúsculas pode permitir `Ana` e `ana` como contas distintas -> comportamento explícito até a equipe definir outra regra.
- O repositório em memória perde os cadastros ao encerrar o processo -> limitação esperada nesta etapa; SQLite e seleção de backend serão implementados separadamente.

## Migration Plan

Não há dados existentes de usuários para migrar. A composição da aplicação começa a oferecer o repositório em memória. A troca pelo modo configurável ocorrerá em etapa posterior.

## Open Questions

Nenhuma para as etapas de modelo, validação e armazenamento em memória selecionadas.
