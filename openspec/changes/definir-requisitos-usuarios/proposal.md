# Definir requisitos de gerenciamento de usuários

## Por quê

As Sprints 1 e 2 pedem cadastro e listagem de usuários, validação de login e senha e seleção entre persistência em memória e persistência permanente. Os casos de uso UC10 e UC11 já existem, mas seus requisitos relacionados estavam inconsistentes e não registravam todos os critérios da sprint.

## O que muda

- Especificar os critérios verificáveis de cadastro e listagem de usuários.
- Documentar os dois modos de persistência e a seleção na inicialização.
- Registrar o tratamento esperado para falhas de leitura e escrita.
- Corrigir a rastreabilidade entre RF06, RF16, RF17 e UC10/UC11.
- Implementar a entidade de usuário, as validações de login e senha, os casos de uso de cadastro/listagem em memória e o repositório SQLite selecionável na inicialização.

## Capacidades

- `gestao-de-usuarios`: cadastro, listagem, validação e persistência de usuários.

## Impacto

- `docs/04-casos-de-uso/uc10.md`
- `docs/04-casos-de-uso/uc11.md`
- `docs/04-casos-de-uso/README.md`
- Especificação OpenSpec de gerenciamento de usuários.
- `nethub/domain/`, `nethub/application/`, `nethub/infrastructure/` e a raiz de composição da aplicação.
- Comandos de interface e diagramas atualizados permanecem etapas futuras.

## Premissas

- A exigência de senha diferente do login vem do enunciado da Sprint 2. A política padrão do AWS IAM exige que a senha não seja idêntica ao nome ou e-mail da conta AWS; as duas regras são relacionadas, mas não equivalentes.
- O segundo mecanismo de persistência pode ser SQLite, conforme permitido no enunciado.
