# Descoberta de rede

## Purpose

Definir a descoberta de hosts em escopo autorizado e a atualização do inventário a partir dos resultados do Nmap.

## Requirements

### Requirement: RF01 — Descobrir dispositivos na rede
O sistema DEVE permitir que o operador execute uma descoberta em um escopo de rede autorizado, usando Nmap e interpretando os resultados XML para atualizar o inventário local SQLite.

**Prioridade:** Essencial
**Casos de uso:** UC01

#### Scenario: Descoberta concluída
- **GIVEN** o operador definiu um escopo autorizado e o Nmap está disponível
- **WHEN** a varredura termina com resultado XML válido
- **THEN** o sistema apresenta os hosts encontrados e atualiza o inventário com a data da descoberta

#### Scenario: Falha ou resultado inválido
- **GIVEN** uma descoberta foi iniciada
- **WHEN** o Nmap falha ou o XML não pode ser interpretado
- **THEN** o sistema informa a falha e não apresenta o resultado como uma descoberta completa
