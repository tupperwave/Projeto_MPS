# 8. Projeto

O projeto da versão 2 está documentado no [Documento do Projeto — Versão 2](09-documento-projeto-v2.md), que consolida o modelo de classes de análise até a Sprint 3, os padrões aplicados e as revisões de código registradas.

## Organização implementada

- `nethub/domain`: entidades, regras de domínio, enums, erros e portas.
- `nethub/application`: serviços de caso de uso e `FacadeSingletonController`.
- `nethub/infrastructure`: adaptadores de persistência SQLite e memória, Nmap, rede local e SSH.
- `nethub/presentation`: CLI e TUI.
- `nethub/bootstrap.py`: composição das dependências e criação do container da aplicação.

## Padrões e responsabilidades

- **Ports and Adapters:** serviços dependem de contratos do domínio; memória, SQLite e ferramentas externas implementam adaptadores.
- **Repository:** separa as operações de persistência de usuários, hosts, anotações e evidências.
- **Facade:** `FacadeSingletonController` apresenta um ponto único de acesso aos gerentes da composição.
- **Singleton por composição:** o escopo de composição define a identidade da fachada, evitando compartilhar instâncias entre aplicações distintas.
- **Injeção de dependência:** `bootstrap.py` monta serviços e repositórios, mantendo decisões de infraestrutura fora dos serviços de caso de uso.
- **Responsabilidade única:** a contagem de entidades está em `EntityCountService`; a fachada delega a operação.

As exceções de domínio e aplicação são derivadas de `NetHubError` quando se destinam a ser apresentadas pela CLI. A documentação de casos de uso e os testes associados complementam as decisões acima; requisitos e padrões ainda não implementados permanecem identificados como pendências no documento versão 2.
