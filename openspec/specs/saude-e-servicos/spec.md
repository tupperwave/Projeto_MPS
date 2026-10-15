# Saúde e serviços

## Purpose

Definir consultas de saúde do host e operações de gerenciamento de serviços systemd.

## Requirements

### Requirement: RF11 — Consultar saúde do host
O sistema DEVE consultar indicadores de disco, memória, carga de CPU e estado de serviços systemd.

**Prioridade:** Importante
**Casos de uso:** UC04

#### Scenario: Métrica indisponível
- **GIVEN** o operador solicita um health check
- **WHEN** uma métrica não pode ser obtida
- **THEN** o sistema a apresenta como indisponível, sem classificá-la como normal

### Requirement: RF12 — Gerenciar serviços systemd
O sistema DEVE permitir consultar serviços e solicitar reinício de serviço remoto, com verificação posterior.

**Prioridade:** Importante
**Casos de uso:** UC05

#### Scenario: Reinício verificado
- **GIVEN** o serviço existe e o operador tem permissão
- **WHEN** o operador confirma o reinício
- **THEN** o sistema solicita o reinício, consulta novamente o estado e registra o resultado
