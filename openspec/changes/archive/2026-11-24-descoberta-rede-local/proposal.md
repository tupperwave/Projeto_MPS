# Proposal

## Why

Hoje o operador precisa conhecer e digitar o CIDR da rede, e a ausência do Nmap só aparece quando a descoberta já foi solicitada. Isso torna o primeiro uso confuso, especialmente em uma instalação nova.

## What Changes

- Permitir que a CLI detecte o CIDR IPv4 associado à rota padrão do sistema.
- Exibir o CIDR detectado e preservar a confirmação explícita antes de varrer.
- Informar como instalar o Nmap no Fedora quando ele não estiver disponível.
- Documentar o pré-requisito e os modos manual e automático.

## Capabilities

### New Capabilities

Nenhuma.

### Modified Capabilities

- `descoberta-de-rede`: permitir a seleção automática da rede local e orientar o operador quando a detecção ou o Nmap não estiverem disponíveis.

## Impact

- CLI, composição do container, adaptador de rede local, especificação RF01 e README.
- Sem novas dependências Python; no Linux, a detecção usa o utilitário `ip` (iproute).
