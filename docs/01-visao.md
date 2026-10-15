# 1. Visão

## 1.1 Propósito

Este repositório documenta o NetHub, projeto da disciplina de Métodos e Projeto de Software. A documentação serve como referência versionada para requisitos, casos de uso e decisões de análise e projeto.

## 1.2 Visão geral

O material atual descreve o NetHub como uma ferramenta terminal-first para descoberta e inventário de hosts, apresentação de ações disponíveis e operação remota com registro de evidências. A proposta cita Nmap, OpenSSH/SFTP e SQLite como tecnologias do MVP.

## 1.3 Problema e motivação

A proposta busca reunir tarefas hoje realizadas com ferramentas e sintaxes distintas em uma interface de terminal, mostrando informações de hosts e operações de infraestrutura em um fluxo único.

## 1.4 Escopo conhecido

As fontes atuais descrevem descoberta de rede, inventário, classificação de capacidades e risco, execução remota, health checks, gestão de serviços systemd, transferência SFTP, workflows, histórico e tags. Os requisitos detalhados e os casos de uso são mantidos em [OpenSpec](../openspec/specs/) e em [casos de uso](04-casos-de-uso/).

## 1.5 Limites e usuários

O material atual aponta como usuário o operador/administrador responsável por selecionar hosts e executar operações. O PDF legado contém escopo e perfis adicionais; esses pontos ainda precisam ser reconciliados com a definição atual do produto antes de serem tratados como linha de base.

## 1.6 Fontes e pendências

A descrição original é curta e o PDF de requisitos tem conteúdo anterior que diverge dos arquivos atuais. Ambos foram preservados em `03-elicitacao/anexos/`. A equipe deve confirmar grafia oficial (`NetHub`/`Nethub`), escopo, usuários e restrições.
