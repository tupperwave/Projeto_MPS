# NetHub — Documento de Requisitos
## 1. Controle do documento
### 1.1. Histórico de versões

| Versão | Data | Descrição |
|---|---|---|
| 0.1 | 2026-10-13 | Versão inicial do documento |
| 1.0 | 2026-10-15 | Inclusão de versionamento, 


##2. Requisitos 
#2.1 REQUISITOS FUNCIONAIS 

## RF01 — Descoberta de rede

O sistema DEVE permitir executar uma varredura em escopo autorizado utilizando Nmap, interpretar o arquivo XML gerado e atualizar o inventário de hosts.

**Prioridade:** Essencial  
**Caso de uso:** UC01
-







| **RF-01** | Descoberta de Rede | Executar varredura via Nmap, ler o arquivo XML gerado e popular os hosts no inventário SQLite. | Essencial |
-
| **RF-02** | Inventário de Hosts |  Classificar e Armazenar dados estruturados dos hosts: IP,SSH, hostname, portas abertas, serviços, timestamps e tags. | Essencial |
-
| **RF-03** | Menu Contextual de Ações | Exibir dinamicamente ações compatíveis com as capacidades detectadas em cada host. | Essencial |
-
| **RF-04** | Modelo de Risco | Associar níveis de risco (Baixo, Médio, Alto) às ações e exigir confirmação proporcional. | Essencial |
-
| **RF-05** | Execução Remota | Executar comandos não interativos via SSH, com geração obrigatória de auditoria. | Essencial |
-
| **RF-7** | Workflow com Dry-Run | Simular ações antes da execução real, apresentando um relatório prévio do que será feito. | Essencial |
-
| **RF-8** | Validação Pós-Ação | Confirmar e homologar o resultado prático da ação executada no ambiente. | Essencial |
-
| **RF-9** | Registro de Evidências | Gravar logs de forma auditavel | Essencial |
-
| **RF-10** | Interface TUI | Oferecer navegação rica via terminal (Text User Interface), com atalhos e feedback visual rápido. | Essencial |
-
| **RF-11** | Health Check | Verificar métricas de disco, memória, carga de CPU e status de serviços Systemd . | Importante |
-
| **RF-12** | Gestão de Serviços Systemd | Consultar status e reiniciar serviços remotos, com verificação automatizada pós-ação. | Importante |
-
| **RF-13** | Transferência de Arquivos | Realizar upload e download de arquivos via SFTP, com validação de integridade por checksum. | Importante |
-
| **RF-14** | Histórico Consultável | Permitir a filtragem e consulta de evidências armazenadas por host, ação executada e período. | Importante |
-
| **RF-15** | Tags de Organização | Permitir a criação e vinculação manual de tags personalizadas para agrupamento de dispositivos. | Desejável |

##Requisitos Não Funcionais
| **RNF-01** | Usabilidade | Interface TUI intuitiva e responsiva 

| **RNF-03** | Segurança | Proteção de credenciais confidenciais

| **RNF-04** | Segurança | Prevenção de execuções acidentais críticas 

| **RNF-06** | Auditabilidade | Integridade e rastreabilidade de dados 

| **RNF-07** | Confiabilidade | Resiliência a falhas de conexão 

| **RNF-08** | Manutenibilidade | Arquitetura de código limpa e modular 

| **RNF-09** | Escalabilidade | Suporte a crescimento de escopo atendendo tempo habil 
| **RNF-10** | Conformidade | Validação legal e autorização de uso 
| **RNF-11** | Evolutividade | A arquitetura deve permitir adicionar capacidades e adaptadores sem exigir reescrever toda a aplicação. 

##Regras de Negócio
| RN-01 | Ações de risco alto ou crítico exigem confirmação explícita.          
| RN-02 | Capacidade "autorizado" só é atribuída quando sudo está disponível.   
| RN-03 | Toda ação executada gera evidência.                                   
| RN-04 | Ações só são exibidas se compatíveis com as capacidades detectadas.   
| RN-05 | Dry-run é obrigatório para ações de risco crítico.                    
| RN-06 | Credenciais não são persistidas sem consentimento explícito.
| RN-07 | Abordagem Agentless, Local-first

