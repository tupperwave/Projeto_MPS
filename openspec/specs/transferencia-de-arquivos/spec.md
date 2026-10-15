# Transferência de arquivos

## Purpose

Definir transferências de arquivos via SFTP e a verificação de integridade quando aplicável.

## Requirements

### Requirement: RF13 — Transferir arquivos via SFTP
O sistema DEVE permitir upload e download via SFTP e verificar integridade por checksum quando aplicável.

**Prioridade:** Importante
**Casos de uso:** UC06

#### Scenario: Transferência íntegra
- **GIVEN** SFTP está disponível e os caminhos têm permissão
- **WHEN** o operador confirma uma transferência
- **THEN** o sistema transfere o arquivo, compara checksums quando aplicável e registra o resultado

#### Scenario: Integridade não confirmada
- **GIVEN** uma transferência foi interrompida ou os checksums divergem
- **WHEN** o sistema conclui a tentativa
- **THEN** o resultado não é apresentado como íntegro e a falha é registrada
