# Relatórios de acesso

## Purpose

Definir o registro e a emissão de estatísticas sobre tentativas de autenticação de usuários.

## Requirements

### Requirement: RF33 — Gerar relatórios de acesso
O sistema DEVE agrupar as tentativas de autenticação por login e gerar estatísticas com total de tentativas, sucessos, falhas e data do último acesso. A aplicação DEVE suportar os formatos texto, HTML e PDF; a CLI DEVE exportar HTML ou PDF para um arquivo informado pelo operador. Os relatórios NÃO DEVEM incluir senhas nem hashes de credenciais.

**Prioridade:** Essencial

**Entrega:** Sprint 4

**Casos de uso:** UC16

#### Scenario: Agrupar acessos por usuário
- **GIVEN** existem tentativas de autenticação bem-sucedidas e malsucedidas
- **WHEN** a aplicação consulta as estatísticas de acesso
- **THEN** retorna uma linha por login com totais, sucessos, falhas e último acesso

#### Scenario: Gerar os formatos disponíveis na aplicação
- **GIVEN** há estatísticas de acesso consultadas
- **WHEN** um cliente solicita um relatório em texto, HTML ou PDF
- **THEN** recebe o conteúdo no formato escolhido com as mesmas estatísticas

#### Scenario: Exportar HTML ou PDF pela CLI
- **GIVEN** o operador informa o formato HTML ou PDF e um caminho de saída
- **WHEN** solicita `relatorio-acessos`
- **THEN** o sistema grava o relatório no caminho informado

#### Scenario: Não expor credenciais
- **GIVEN** existem usuários e tentativas de autenticação registradas
- **WHEN** o sistema gera um relatório
- **THEN** o conteúdo não inclui senhas, salts nem hashes de senha

#### Scenario: Não há tentativas registradas
- **GIVEN** o repositório de acessos está vazio
- **WHEN** o sistema gera um relatório
- **THEN** gera o cabeçalho e nenhuma linha de usuário
