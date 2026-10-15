# Operações remotas

## Purpose

Definir a execução de comandos remotos via SSH e a validação dos resultados pós-ação.

## Requirements

### Requirement: RF05 — Executar comando remoto
O sistema DEVE permitir a execução não interativa de comandos via SSH em host autorizado e registrar a tentativa.

**Prioridade:** Essencial
**Casos de uso:** UC03

#### Scenario: Comando executado
- **GIVEN** o host está alcançável, a sessão SSH foi autenticada e a operação foi autorizada
- **WHEN** o operador confirma o comando
- **THEN** o sistema executa com timeout e apresenta saída, erro e código de saída, registrando o resultado

#### Scenario: Autenticação ou conexão falha
- **GIVEN** o operador solicitou um comando remoto
- **WHEN** autenticação ou conexão falha
- **THEN** o sistema informa e registra a falha sem revelar credenciais

### Requirement: RF08 — Validar resultado pós-ação
O sistema DEVE permitir confirmar o resultado prático da ação executada.

**Prioridade:** Essencial
**Casos de uso:** UC05, UC06, UC07

#### Scenario: Estado esperado não confirmado
- **GIVEN** uma ação foi executada e possui uma verificação configurada
- **WHEN** a verificação não confirma o estado esperado
- **THEN** o sistema destaca que a execução não equivale a sucesso validado
