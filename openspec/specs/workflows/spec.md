# Workflows

## Purpose

Definir a simulação de workflows antes de executar suas etapas.

## Requirements

### Requirement: RF07 — Simular workflow antes da execução
O sistema DEVE permitir simular as etapas de um workflow e apresentar previamente as ações planejadas.

**Prioridade:** Essencial
**Casos de uso:** UC07

#### Scenario: Simular sem executar
- **GIVEN** o operador selecionou um workflow e um host-alvo
- **WHEN** solicita um dry-run
- **THEN** o sistema apresenta etapas, riscos e pré-condições sem alterar o alvo

#### Scenario: Cancelar simulação
- **GIVEN** o dry-run foi apresentado
- **WHEN** o operador cancela
- **THEN** nenhuma etapa de alteração é executada
