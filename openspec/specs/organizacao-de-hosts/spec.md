# Organização de hosts

## Purpose

Definir a criação e associação de tags para organizar hosts no inventário.

## Requirements

### Requirement: RF15 — Organizar hosts com tags
O sistema DEVE permitir criar tags personalizadas, associá-las manualmente a hosts e filtrar o inventário por tag.

**Prioridade:** Desejável
**Casos de uso:** UC09

#### Scenario: Associar tag
- **GIVEN** há um host no inventário
- **WHEN** o operador associa uma tag válida
- **THEN** o sistema salva a associação e permite localizar hosts por essa tag
