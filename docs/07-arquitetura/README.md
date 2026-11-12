# 7. Arquitetura

O diagrama de classes atual sugere separação entre apresentação, aplicação, domínio, portas e adaptadores de infraestrutura. O modelo complementar de gerenciamento de usuários está em [`05-analise.md`](../05-analise.md#modelo-de-classes--gerenciamento-de-usuários) e mostra a seleção entre RAM e SQLite, além dos erros de domínio. Os diagramas lógico e físico e a relação entre requisitos não funcionais e decisões arquiteturais ainda precisam ser elaborados.

Decisões aprovadas devem ser registradas em `adr/` com identificadores `ADR-nnnn`.



## Camadas do sistema

### Domínio

Localizado em `nethub/domain`.

Contém:

- entidades do sistema;
- objetos de valor;
- regras de validação;
- enums;
- exceções de domínio;
- interfaces ou portas utilizadas pela aplicação.

As portas de persistência são definidas no domínio, mas suas implementações
ficam na infraestrutura. Dessa forma, o domínio define o contrato sem depender
do SQLite ou de outra tecnologia específica.

### Business ou aplicação

Localizada em `nethub/application`.

Contém os serviços responsáveis pelos casos de uso do sistema, como:

- cadastro e consulta de usuários;
- autenticação;
- registro de acessos;
- geração de estatísticas;
- geração de relatórios;
- descoberta de hosts;
- execução de comandos;
- gerenciamento de anotações;
- consulta do histórico de operações.

A camada business depende das interfaces definidas no domínio, e não das
classes concretas de infraestrutura.

### Infraestrutura

Localizada em `nethub/infrastructure`.

Contém as implementações concretas dos contratos definidos pelo domínio, como:

- repositórios SQLite;
- repositório de usuários em memória;
- conexão e migrações do banco;
- Adapter do logging;
- Adapter do Nmap;
- Adapter do SSH.

### Apresentação

Localizada em `nethub/presentation`.

Contém:

- interface de linha de comando;
- interface de terminal com `curses`;
- confirmação de ações;
- seleção do formato dos relatórios.

### Composição

A composição das dependências é realizada em `nethub/bootstrap.py`.

Esse módulo cria as implementações concretas e injeta essas dependências nos
serviços da aplicação.

## Padrões de projeto aplicados

### Repository

Os contratos de persistência estão definidos em `nethub/domain/ports.py`.

São utilizados os seguintes repositórios:

- `UserRepository`;
- `HostRepository`;
- `EvidenceRepository`;
- `HostAnnotationRepository`;
- `UserAccessRepository`.

As implementações concretas ficam na infraestrutura:

- `SqliteUserRepository`;
- `MemoryUserRepository`;
- `SqliteHostRepository`;
- `SqliteEvidenceRepository`;
- `SqliteHostAnnotationRepository`;
- `SqliteUserAccessRepository`.

A camada business utiliza esses contratos sem conhecer comandos SQL,
tabelas ou detalhes do SQLite.

### Abstract Factory

A classe `RepositoryFactory`, localizada em
`nethub/infrastructure/repository_factory.py`, cria uma família de
repositórios relacionados.

Ela centraliza a seleção das implementações concretas, incluindo a escolha
entre:

- `MemoryUserRepository`;
- `SqliteUserRepository`.

A aplicação recebe o objeto `Repositories` já montado e não precisa instanciar
diretamente cada repositório.

### Adapter

O `PythonLoggingAdapter` adapta a API de `logging.Logger` à interface
`LoggerPort` utilizada pela aplicação.

O fluxo é:

```text
UserService
    -> LoggerPort
    -> PythonLoggingAdapter
    -> logging.Logger