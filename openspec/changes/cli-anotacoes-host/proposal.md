# Proposal

## Why

O CRUD de anotações existe no serviço de aplicação, mas o operador não consegue executá-lo diretamente pela CLI, o que dificulta demonstrar e utilizar a funcionalidade da Sprint 3.

## What Changes

- Expor criação, consulta, listagem, atualização e exclusão de anotações em subcomandos da CLI.
- Encaminhar as operações pela fachada única dos gerentes.
- Atualizar o UC15, o diagrama de classes e o README com os comandos disponíveis.

## Capabilities

### New Capabilities

- `anotacoes-host-cli`: operações de anotações de hosts disponíveis ao operador pela linha de comando.

### Modified Capabilities

Nenhuma.

## Impact

- Parser e dispatch da CLI, documentação de uso, UC15 e diagrama de análise.
- Sem novas dependências.
