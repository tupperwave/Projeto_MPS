# Inventário e capacidades

## Purpose

Definir a manutenção dos dados de hosts, a apresentação de capacidades e a confirmação proporcional ao risco.

## Requirements

### Requirement: RF02 — Manter inventário de hosts
O sistema DEVE armazenar dados estruturados dos hosts, incluindo endereço IP, hostname, portas, serviços, timestamps e tags disponíveis.

**Prioridade:** Essencial
**Casos de uso:** UC01, UC02, UC09

#### Scenario: Consultar host conhecido
- **GIVEN** um host foi registrado no inventário
- **WHEN** o operador consulta esse host
- **THEN** o sistema apresenta os dados armazenados e a última observação disponível

### Requirement: RF03 — Exibir ações compatíveis
O sistema DEVE exibir ações de acordo com as capacidades detectadas para o host.

**Prioridade:** Essencial
**Casos de uso:** UC02

#### Scenario: Capacidade desconhecida
- **GIVEN** uma capacidade do host não foi verificada
- **WHEN** o operador consulta as ações disponíveis
- **THEN** o sistema não apresenta a capacidade desconhecida como confirmada

### Requirement: RF04 — Aplicar modelo de risco
O sistema DEVE associar níveis de risco às ações e solicitar confirmação proporcional ao risco.

**Prioridade:** Essencial
**Casos de uso:** UC03, UC05, UC06, UC07

#### Scenario: Ação de risco alto
- **GIVEN** uma ação foi classificada como de risco alto
- **WHEN** o operador solicita sua execução
- **THEN** o sistema exige confirmação explícita antes de executar
