## MODIFIED Requirements

### Requirement: RF01 — Descobrir dispositivos na rede
O sistema DEVE permitir que o operador execute uma descoberta em um escopo de rede autorizado, usando Nmap e interpretando os resultados XML para atualizar o inventário local SQLite. O escopo pode ser informado como CIDR ou detectado automaticamente a partir da interface associada à rota padrão IPv4. Na detecção automática, o sistema DEVE exibir o CIDR selecionado e obter confirmação do operador antes da varredura, salvo quando a opção explícita de autorização já tiver sido informada. Se o alvo local não puder ser detectado, o sistema DEVE orientar o operador a informar um CIDR manualmente. Se Nmap estiver ausente, o sistema DEVE informar como instalá-lo no Fedora.

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

#### Scenario: Descoberta automática da rede local
- **GIVEN** a máquina tem uma rota padrão IPv4, um endereço IPv4 global na interface dessa rota e o Nmap está disponível
- **WHEN** o operador solicita descoberta da rede local
- **THEN** o sistema mostra o CIDR detectado, pede autorização quando ela não foi antecipada, e só então inicia a varredura

#### Scenario: Rede local não detectável
- **GIVEN** a rota padrão IPv4 ou o endereço da interface não pode ser consultado
- **WHEN** o operador solicita descoberta automática
- **THEN** o sistema informa que não detectou a rede e orienta o uso de um CIDR manual

#### Scenario: Nmap ausente
- **GIVEN** o Nmap não está disponível no PATH
- **WHEN** o operador solicita uma descoberta
- **THEN** o sistema informa a instalação do Nmap no Fedora sem iniciar a varredura
