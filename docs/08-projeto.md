# 8. Projeto

O projeto está documentado no [Documento do Projeto — Versão 2.1](09-documento-projeto-v2.md), que consolida os padrões aplicados até a Sprint 5 e as revisões de código registradas.

## Organização implementada

- `nethub/domain`: entidades, regras de domínio, enums, erros e portas.
- `nethub/application`: serviços de caso de uso e `FacadeSingletonController`.
- `nethub/infrastructure`: adaptadores de persistência SQLite e memória, Nmap, rede local e SSH.
- `nethub/presentation`: CLI e TUI.
- `nethub/bootstrap.py`: composição das dependências e criação do container da aplicação.

## Padrões e responsabilidades

- **Ports and Adapters:** serviços dependem de contratos do domínio; memória, SQLite e ferramentas externas implementam adaptadores.
- **Repository:** separa as operações de persistência de usuários, hosts, anotações e evidências por portas no domínio.
- **Abstract Factory (parcial):** `RepositoryFactory` monta o conjunto de repositórios e seleciona memória ou SQLite somente para usuários; os outros repositórios são SQLite.
- **Facade:** `FacadeSingletonController` apresenta operações de anotações e contagem sem exigir que a apresentação acesse diretamente os gerentes.
- **Command:** `FacadeSingletonController` é o cliente que cria os comandos; `Command` define o contrato, os comandos concretos encapsulam as operações, `CommandInvoker` os executa e `HostAnnotationService` recebe e realiza os pedidos.
- **Memento:** `AnotacaoHost` cria e restaura `MementoAnotacao`; o serviço mantém somente o snapshot anterior à última atualização válida, e o repositório SQLite permite desfazê-la em uma execução separada da CLI.
- **Template Method:** `AccessReportTemplate` fixa o fluxo para os relatórios de acesso em texto, HTML e PDF.
- **Adapter de logging:** `PythonLoggingAdapter` implementa `LoggerPort` sobre o logging da biblioteca padrão.
- **Singleton por composição (parcial):** `FacadeSingletonController` mantém uma instância por `FacadeSingletonScope`; composições distintas podem ter fachadas diferentes. Se a rubrica exigir uma única instância global do processo, este critério ainda não está satisfeito.
- **Adapter:** `NmapAdapter`, `SshExecutor` e `LinuxLocalNetworkDetector` adaptam ferramentas e APIs do sistema às portas usadas pela aplicação.
- **Injeção de dependência:** `bootstrap.py` monta serviços e repositórios, mantendo decisões de infraestrutura fora dos serviços de caso de uso.
- **Responsabilidade única:** a contagem de entidades está em `EntityCountService`; a fachada delega a operação.

As exceções de domínio e aplicação são derivadas de `NetHubError` quando se destinam a ser apresentadas pela CLI. A documentação de casos de uso e os testes associados complementam as decisões acima; requisitos e padrões ainda não implementados permanecem identificados como pendências no documento versão 2.1.

Os relatórios de acesso consultam as estatísticas persistidas e aplicam os formatos pelo Template Method; a documentação de arquitetura detalha também as fábricas e adaptadores incorporados da `main`.
