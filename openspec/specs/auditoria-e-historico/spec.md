# Auditoria e histórico

### Requirement: RF09 — Registrar evidências
O sistema DEVE registrar evidências auditáveis das ações executadas.

**Prioridade:** Essencial
**Casos de uso:** UC03, UC04, UC05, UC06, UC07

#### Scenario: Ação concluída ou falha
- **GIVEN** uma ação foi tentada
- **WHEN** ela termina, falha ou é cancelada
- **THEN** o sistema registra o resultado correspondente e suas evidências disponíveis

### Requirement: RF14 — Consultar histórico
O sistema DEVE permitir filtrar registros por host, ação e período.

**Prioridade:** Importante
**Casos de uso:** UC08

#### Scenario: Consultar registros
- **GIVEN** existem registros no histórico
- **WHEN** o operador aplica filtros de host, ação ou período
- **THEN** o sistema apresenta os registros correspondentes
