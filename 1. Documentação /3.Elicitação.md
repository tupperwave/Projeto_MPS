# NetHub — Documento de Requisitos
###Versão - 

##Casos de Uso
# NetHub — Casos de Uso

**Versão:** 0.1  
**Data:** 2026-10-13  
**Autores:** André Luiz Rodrigues Castro da Nóbrega, Marcos José Edu Ekua Ayingono, Lucas Martins de Lima Oliveira

---

## UC-01 — Descobrir dispositivos na rede

| Campo | Descrição |
|-------|-----------|
| **Ator** | Administrador/operador |
| **Objetivo** | Identificar hosts em rede autorizada e registrar no inventário |
| **Pré-condições** | Autorização de varredura; Nmap disponível; intervalo definido |

**Fluxo principal**
1. Operador abre a descoberta.
2. Sistema solicita/apresenta o intervalo de rede.
3. Operador confirma o alvo.
4. NetHub executa Nmap.
5. Sistema interpreta e apresenta os hosts.
6. Sistema grava/atualiza o inventário SQLite.

**Exceções**
- Intervalo inválido → sistema solicita correção.
- Nmap indisponível ou falha → resultado não é apresentado como completo.
- Nenhum host encontrado → sistema informa varredura sem resultados.

**Pós-condição:** resultados válidos no inventário, com data da descoberta.

---

## UC-02 — Consultar host e capacidades

| Campo | Descrição |
|-------|-----------|
| **Ator** | Administrador/operador |
| **Objetivo** | Ver informações e ações disponíveis para um host |
| **Pré-condições** | Host no inventário |

**Fluxo principal**
1. Operador abre a lista de hosts.
2. Seleciona um dispositivo.
3. Sistema apresenta dados, serviços e última descoberta.
4. NetHub apresenta capacidades e ações compatíveis.
5. Operador seleciona uma capacidade.

**Exceções**
- Dados desatualizados → sistema indica a última vez visto.
- Capacidade não verificada → sistema informa como desconhecida.

**Pós-condição:** estado conhecido e ações disponíveis consultáveis.

---

## UC-03 — Executar comando remoto

| Campo | Descrição |
|-------|-----------|
| **Ator** | Administrador/operador |
| **Objetivo** | Executar comando não interativo em host autorizado |
| **Pré-condições** | Host alcançável; SSH autenticado; autorização do operador |

**Fluxo principal**
1. Operador seleciona host e opção de comando remoto.
2. Sistema identifica o alvo e solicita o comando.
3. NetHub apresenta alvo e detalhes da operação.
4. Operador confirma conforme o risco.
5. Sistema executa via SSH com timeout.
6. Sistema apresenta saída, erro e exit code.
7. Sistema registra a operação na auditoria.

**Exceções**
- Falha de autenticação → informada sem revelar credenciais.
- Conexão expirada → falha registrada.
- Cancelamento → nada é executado.

**Pós-condição:** tentativa registrada, concluída ou não.

---

## UC-04 — Consultar saúde do host

| Campo | Descrição |
|-------|-----------|
| **Ator** | Administrador/operador |
| **Objetivo** | Obter indicadores básicos de funcionamento |
| **Pré-condições** | Host no inventário; método de consulta remota disponível |

**Fluxo principal**
1. Operador seleciona host.
2. Escolhe verificação de saúde.
3. Sistema consulta disco, memória, carga e systemd.
4. NetHub apresenta valores e destaca limites excedidos.
5. Sistema registra horário e resultado.

**Exceções**
- Métrica indisponível → exibida como indisponível, não como normal.
- Falha de conexão → apresentada e registrada.

**Pós-condição:** diagnóstico básico com os dados obtidos.

---

## UC-05 — Reiniciar serviço systemd

| Campo | Descrição |
|-------|-----------|
| **Ator** | Administrador/operador |
| **Objetivo** | Reiniciar serviço remoto e verificar estado posterior |
| **Pré-condições** | systemd no host; SSH autenticado; permissão de reinício |

**Fluxo principal**
1. Operador consulta os serviços disponíveis.
2. Escolhe o serviço a reiniciar.
3. Sistema apresenta host, serviço e risco.
4. Operador confirma.
5. NetHub solicita o reinício.
6. Sistema consulta novamente o estado.
7. Sistema apresenta resultado e verificação.
8. NetHub registra execução e evidências.

**Exceções**
- Serviço inexistente → não se afirma reinício bem-sucedido.
- Permissão insuficiente → falha registrada.
- Serviço não volta ao esperado → falha de validação destacada.

**Pós-condição:** tentativa e verificação registradas.

---

## UC-06 — Transferir arquivo por SFTP

| Campo | Descrição |
|-------|-----------|
| **Ator** | Administrador/operador |
| **Objetivo** | Enviar ou baixar arquivo entre local e host remoto |
| **Pré-condições** | SFTP disponível; autenticação configurada; permissão no caminho |

**Fluxo principal**
1. Operador seleciona host e transferência.
2. Escolhe upload ou download.
3. Informa caminhos local e remoto.
4. Sistema apresenta origem, destino e operação.
5. Operador confirma.
6. NetHub executa a transferência.
7. Sistema compara checksums, quando aplicável.
8. Sistema apresenta resultado e registra.

**Exceções**
- Caminho inválido ou sem permissão → falha informada.
- Transferência interrompida → registrada como incompleta.
- Checksums divergentes → integridade não confirmada.

**Pós-condição:** resultado e verificação de integridade registrados.

---

## UC-07 — Simular e executar workflow

| Campo | Descrição |
|-------|-----------|
| **Ator** | Administrador/operador |
| **Objetivo** | Revisar etapas antes de executar operação composta |
| **Pré-condições** | Workflow definido; alvos e etapas reconhecidos |

**Fluxo principal**
1. Operador seleciona workflow e host-alvo.
2. NetHub verifica capacidades e pré-condições.
3. Sistema apresenta etapas, riscos e verificações (dry-run).
4. Operador decide cancelar ou continuar.
5. Se confirmado, sistema executa as etapas na ordem.
6. Sistema valida cada etapa com verificação configurada.
7. NetHub apresenta resumo (concluídas, falhas, não executadas).
8. Sistema registra execução e evidências.

**Exceções**
- Pré-condição obrigatória falha → execução bloqueada ou decisão prevista.
- Etapa falha → comportamento previamente definido (interromper/continuar/compensar).
- Cancelamento no dry-run → nenhuma etapa executada.

**Pós-condição:** plano simulado e, se autorizado, resultados reais registrados.

---

## UC-08 — Consultar histórico de operações

| Campo | Descrição |
|-------|-----------|
| **Ator** | Administrador/operador |
| **Objetivo** | Localizar e revisar ações anteriores |
| **Pré-condições** | Registros locais existentes |

**Fluxo principal**
1. Operador abre o histórico.
2. Define filtros (host, ação, resultado, período).
3. Sistema apresenta registros correspondentes.
4. Operador abre um registro para detalhes e evidências.

**Exceções**
- Nenhum registro encontrado → sistema informa ausência de resultados.

**Pós-condição:** histórico consultável sem alterar registros.

---

## UC-09 — Organizar hosts com tags

| Campo | Descrição |
|-------|-----------|
| **Ator** | Administrador/operador |
| **Objetivo** | Agrupar hosts para facilitar identificação e consulta |
| **Pré-condições** | Pelo menos um host no inventário |

**Fluxo principal**
1. Operador seleciona um host.
2. Cria ou escolhe uma tag existente.
3. Associa a tag ao host.
4. Sistema salva e permite filtrar hosts por tag.

**Exceções**
- Tag vazia ou inválida → sistema solicita correção.

**Pós-condição:** tag associada e disponível para filtragem.
##Requisitos funcionais
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

