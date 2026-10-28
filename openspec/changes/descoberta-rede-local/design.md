# Design

## Context

RF01 já recebe um alvo validado e exige autorização no serviço. A melhoria deve simplificar a escolha do alvo sem remover essa proteção ou acrescentar dependências Python.

## Decisions

- A CLI aceitará um CIDR posicional ou `--rede-local`, exatamente uma das opções.
- Um adaptador de infraestrutura consultará `ip -4 route show default`, encontrará a interface padrão e obterá seu primeiro endereço IPv4 global com prefixo. `ipaddress` converterá esse endereço em CIDR.
- O detector será injetado pelo bootstrap e exposto no container, mantendo subprocessos fora da apresentação.
- A CLI exibirá o CIDR detectado e reutilizará a confirmação já existente. `--autorizado` permanece como autorização explícita do operador.
- Falhas do utilitário `ip`, da rota ou do endereço resultarão em mensagem de domínio e instrução para informar o alvo manualmente.
- A ausência do Nmap continuará sendo uma falha tratada e receberá uma mensagem de instalação para Fedora (`sudo dnf install nmap`). O programa não instalará pacotes do sistema automaticamente.

## Risks

- A interface da rota padrão pode ser VPN ou túnel; a CLI mostra o CIDR e pede autorização para o operador poder confirmar o escopo.
- Sistemas sem `iproute` não terão detecção automática, mas continuarão podendo usar um CIDR manual.
- Redes acima do limite de endereços existente continuarão sendo rejeitadas pelo validador `AlvoRede`.
