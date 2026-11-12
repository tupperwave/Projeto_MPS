classDiagram
    direction LR

    %% =========================================================
    %% DOMINIO
    %% =========================================================

    namespace Dominio {
        class Usuario {
            <<Entity>>
            +login: str
            -_sal: bytes
            -_hash_senha: bytes
            +__init__(login, senha)
            +restaurar(login, sal, hash_senha) Usuario
            +verificar_senha(senha) bool
        }

        class Host {
            <<Entity>>
            +id: int
            +ip: str
            +hostname: str
            +porta_ssh: int
            +portas: list
            +capacidades: list
            +tags: set
            +salvar()
            +adicionar_tag(tag)
            +remover_tag(tag)
            +possui_capacidade(tipo) bool
            +esta_desatualizado() bool
        }

        class AnotacaoHost {
            <<Entity>>
            +id: int
            +host_ip: str
            +conteudo: str
            +criada_em: datetime
            +atualizada_em: datetime
            +atualizar(conteudo)
        }

        class Evidencia {
            <<Entity>>
            +id: int
            +tipo_acao: TipoAcao
            +descricao: str
            +alvo: str
            +risco: NivelRisco
            +status: StatusExecucao
            +timestamp: datetime
            +hash_anterior: str
            +hash_integridade: str
            +calcular_hash() str
            +selar(hash_anterior)
            +integra() bool
        }

        class AcessoUsuario {
            <<Entity>>
            +login: str
            +ocorrido_em: datetime
            +sucesso: bool
        }

        class EstatisticaAcesso {
            <<Value Object>>
            +login: str
            +total: int
            +sucessos: int
            +falhas: int
            +ultimo_acesso: datetime
        }

        class AlvoRede {
            <<Value Object>>
            +cidr: str
            +criar(texto) AlvoRede
            +total_enderecos: int
        }

        class ResultadoExecucao {
            <<Value Object>>
            +status: StatusExecucao
            +stdout: str
            +stderr: str
            +exit_code: int
            +duracao_s: float
            +sucesso: bool
        }

        class FiltroHistorico {
            <<Value Object>>
            +alvo: str
            +tipo_acao: TipoAcao
            +status: StatusExecucao
            +de: datetime
            +ate: datetime
            +limite: int
        }
    }

    %% =========================================================
    %% PORTAS E CONTRATOS
    %% =========================================================

    namespace Portas {
        class UserRepository {
            <<Repository Port>>
            +salvar(usuario) Usuario
            +buscar_por_login(login) Usuario
            +listar() list
        }

        class UserAccessRepository {
            <<Repository Port>>
            +registrar(acesso) AcessoUsuario
            +estatisticas() list
        }

        class HostRepository {
            <<Repository Port>>
            +salvar(host) Host
            +buscar_por_ip(ip) Host
            +listar(tag) list
        }

        class EvidenceRepository {
            <<Repository Port>>
            +salvar(evidencia) Evidencia
            +ultimo_hash() str
            +buscar(filtro) list
            +buscar_por_id(id) Evidencia
            +todas_em_ordem() Iterator
        }

        class HostAnnotationRepository {
            <<Repository Port>>
            +salvar(anotacao) AnotacaoHost
            +buscar_por_id(id) AnotacaoHost
            +listar(host_ip) list
            +excluir(id) bool
        }

        class LoggerPort {
            <<Port>>
            +info(mensagem, dados)
            +debug(mensagem, dados)
            +warning(mensagem, dados)
            +error(mensagem, dados)
        }

        class NetworkScannerPort {
            <<Port>>
            +disponivel() bool
            +varrer(alvo) list
        }

        class LocalNetworkDetectorPort {
            <<Port>>
            +detectar() AlvoRede
        }

        class RemoteExecutorPort {
            <<Port>>
            +executar(host, comando, timeout_s) ResultadoExecucao
        }

        class ConfirmationPort {
            <<Port>>
            +confirmar(pedido) bool
        }

        class EntityCountSource {
            <<Port>>
            +contar() int
        }
    }

    %% =========================================================
    %% BUSINESS / APPLICATION
    %% =========================================================

    namespace Business {
        class UserService {
            <<Business Service>>
            -_repositorio: UserRepository
            -_repositorio_acessos: UserAccessRepository
            -_logger: LoggerPort
            +cadastrar(login, senha) Usuario
            +listar() list
            +autenticar(login, senha) bool
            +registrar_acesso(login, sucesso) AcessoUsuario
        }

        class UserAccessReportService {
            <<Business Service>>
            -_repositorio: UserAccessRepository
            +gerar(formato) bytes
        }

        class AccessReportTemplate {
            <<Template Method>>
            +gerar(estatisticas) bytes
            +iniciar_documento()
            +escrever_titulo(titulo)
            +escrever_linha(estatistica)
            +finalizar_documento()
            +resultado() bytes
        }

        class PlainTextAccessReport {
            <<Concrete Template>>
            +iniciar_documento()
            +escrever_titulo(titulo)
            +escrever_linha(estatistica)
            +finalizar_documento()
            +resultado() bytes
        }

        class HtmlAccessReport {
            <<Concrete Template>>
            +iniciar_documento()
            +escrever_titulo(titulo)
            +escrever_linha(estatistica)
            +finalizar_documento()
            +resultado() bytes
        }

        class PdfAccessReport {
            <<Concrete Template>>
            +iniciar_documento()
            +escrever_titulo(titulo)
            +escrever_linha(estatistica)
            +finalizar_documento()
            +resultado() bytes
        }

        class InventoryService {
            <<Business Service>>
            +listar_hosts(tag) list
            +obter_host(ip) Host
            +salvar_host(host) Host
        }

        class DiscoveryService {
            <<Business Service>>
            +descobrir(alvo) ResultadoDescoberta
        }

        class EvidenceService {
            <<Business Service>>
            +registrar(evidencia) Evidencia
        }

        class HistoryService {
            <<Business Service>>
            +consultar(filtro) list
            +obter(id) Evidencia
            +verificar_integridade() bool
        }

        class HostAnnotationService {
            <<Business Service>>
            +criar(host_ip, conteudo) AnotacaoHost
            +obter(id) AnotacaoHost
            +listar(host_ip) list
            +atualizar(id, conteudo) AnotacaoHost
            +excluir(id) bool
        }

        class ActionExecutor {
            <<Business Service>>
            +executar(host, comando, timeout_s)
            +simular(host, comando)
        }

        class CapabilityDetector {
            <<Business Service>>
            +detectar(host) Host
        }

        class EntityCountService {
            <<Business Service>>
            +quantidade_entidades_cadastradas() int
        }

        class ActionRegistry {
            <<Application Component>>
            +registrar(acao)
            +obter(nome)
        }

        class ContextMenuService {
            <<Business Service>>
            +listar_acoes()
        }

        class GerentesController {
            <<Controller Bundle>>
            +inventario: InventoryService
            +descoberta: DiscoveryService
            +executor: ActionExecutor
            +evidencias: EvidenceService
            +historico: HistoryService
            +usuarios: UserService
            +anotacoes: HostAnnotationService
        }

        class FacadeSingletonScope {
            <<Composition Scope>>
        }

        class FacadeSingletonController {
            <<Facade>>
            +gerentes: GerentesController
            +quantidade_entidades_cadastradas() int
        }

        class ResultadoDescoberta {
            <<Value Object>>
            +alvo: AlvoRede
            +hosts: list
        }
    }

    %% =========================================================
    %% INFRAESTRUTURA
    %% =========================================================

    namespace Infraestrutura {
        class RepositoryFactory {
            <<Abstract Factory>>
            -_banco: SqliteDatabase
            -_persistencia_usuarios: str
            +criar() Repositories
            -_criar_usuarios() UserRepository
        }

        class Repositories {
            <<Product Family>>
            +hosts: HostRepository
            +evidencias: EvidenceRepository
            +usuarios: UserRepository
            +anotacoes: HostAnnotationRepository
            +acessos: UserAccessRepository
        }

        class SqliteDatabase {
            <<Infrastructure Service>>
            +executar(sql, parametros)
            +consultar(sql, parametros)
            +fechar()
        }

        class MemoryUserRepository {
            <<Concrete Repository>>
            +salvar(usuario) Usuario
            +buscar_por_login(login) Usuario
            +listar() list
            +contar() int
        }

        class SqliteUserRepository {
            <<Concrete Repository>>
            +salvar(usuario) Usuario
            +buscar_por_login(login) Usuario
            +listar() list
            +contar() int
        }

        class SqliteHostRepository {
            <<Concrete Repository>>
            +salvar(host) Host
            +buscar_por_ip(ip) Host
            +listar(tag) list
            +contar() int
        }

        class SqliteEvidenceRepository {
            <<Concrete Repository>>
            +salvar(evidencia) Evidencia
            +ultimo_hash() str
            +buscar(filtro) list
            +buscar_por_id(id) Evidencia
            +todas_em_ordem() Iterator
            +contar() int
        }

        class SqliteHostAnnotationRepository {
            <<Concrete Repository>>
            +salvar(anotacao) AnotacaoHost
            +buscar_por_id(id) AnotacaoHost
            +listar(host_ip) list
            +excluir(id) bool
            +contar() int
        }

        class SqliteUserAccessRepository {
            <<Concrete Repository>>
            +registrar(acesso) AcessoUsuario
            +estatisticas() list
        }

        class PythonLoggingAdapter {
            <<Adapter>>
            -_logger: logging.Logger
            +info(mensagem, dados)
        }

        class NmapAdapter {
            <<Adapter>>
            +disponivel() bool
            +varrer(alvo) list
        }

        class SshExecutor {
            <<Adapter>>
            +executar(host, comando, timeout_s) ResultadoExecucao
        }

        class LinuxLocalNetworkDetector {
            <<Adapter>>
            +detectar() AlvoRede
        }

        class PythonLogging {
            <<External Library>>
            +info(mensagem, dados)
        }

        class Nmap {
            <<External Tool>>
            +scan(alvo)
        }

        class SSH {
            <<External Tool>>
            +execute(host, comando)
        }
    }

    %% =========================================================
    %% APRESENTACAO
    %% =========================================================

    namespace Apresentacao {
        class CLI {
            <<Presentation>>
            +main()
            +usuarios()
            +relatorio_acessos()
            +anotacoes()
        }

        class TUI {
            <<Presentation>>
            +iniciar()
            +exibir_hosts()
            +exibir_usuarios()
            +exibir_historico()
        }

        class CliConfirmacao {
            <<Adapter de Apresentacao>>
            +confirmar(pedido) bool
        }

        class TuiConfirmacao {
            <<Adapter de Apresentacao>>
            +confirmar(pedido) bool
        }
    }

    %% =========================================================
    %% COMPOSICAO E DEPENDENCIAS PRINCIPAIS
    %% =========================================================

    class Container {
        <<Composition Root>>
        +banco: SqliteDatabase
        +usuarios: UserService
        +relatorios_acesso: UserAccessReportService
        +anotacoes: HostAnnotationService
        +facade: FacadeSingletonController
        +fechar()
    }

    Container --> RepositoryFactory : cria
    Container --> Repositories : recebe
    Container --> SqliteDatabase : inicializa
    Container --> UserService : injeta dependencias
    Container --> UserAccessReportService : injeta dependencias
    Container --> PythonLoggingAdapter : cria
    Container --> NmapAdapter : cria
    Container --> SshExecutor : cria
    Container --> FacadeSingletonController : compoe

    %% =========================================================
    %% RELACIONAMENTOS DO DOMINIO
    %% =========================================================

    Host "1" --> "0..*" AnotacaoHost : possui
    Host "1" --> "0..*" Dominio.PortaServico : contem
    Host "1" --> "0..*" Dominio.Capacidade : possui
    Evidencia --> FiltroHistorico : filtrada por
    AcessoUsuario --> Usuario : identifica por login
    EstatisticaAcesso --> Usuario : resume acessos

    %% =========================================================
    %% RELACIONAMENTOS DA BUSINESS
    %% =========================================================

    UserService --> UserRepository : consulta e salva usuarios
    UserService --> UserAccessRepository : registra tentativas
    UserService --> LoggerPort : registra eventos

    UserAccessReportService --> UserAccessRepository : consulta estatisticas
    UserAccessReportService --> AccessReportTemplate : usa formato

    AccessReportTemplate <|-- PlainTextAccessReport
    AccessReportTemplate <|-- HtmlAccessReport
    AccessReportTemplate <|-- PdfAccessReport

    InventoryService --> HostRepository
    InventoryService --> CapabilityDetector
    DiscoveryService --> NetworkScannerPort
    DiscoveryService --> InventoryService
    DiscoveryService --> EvidenceService
    EvidenceService --> EvidenceRepository
    HistoryService --> EvidenceRepository
    HostAnnotationService --> HostAnnotationRepository
    HostAnnotationService --> HostRepository
    ActionExecutor --> RemoteExecutorPort
    ActionExecutor --> EvidenceService
    EntityCountService --> EntityCountSource

    GerentesController --> InventoryService
    GerentesController --> DiscoveryService
    GerentesController --> ActionExecutor
    GerentesController --> EvidenceService
    GerentesController --> HistoryService
    GerentesController --> UserService
    GerentesController --> HostAnnotationService

    FacadeSingletonController --> GerentesController
    FacadeSingletonController --> EntityCountService
    FacadeSingletonScope ..> FacadeSingletonController : uma instancia por composicao

    %% =========================================================
    %% RELACIONAMENTOS DOS REPOSITORIES
    %% =========================================================

    RepositoryFactory --> Repositories : cria familia

    Repositories --> HostRepository
    Repositories --> EvidenceRepository
    Repositories --> UserRepository
    Repositories --> HostAnnotationRepository
    Repositories --> UserAccessRepository

    MemoryUserRepository ..|> UserRepository
    SqliteUserRepository ..|> UserRepository
    SqliteHostRepository ..|> HostRepository
    SqliteEvidenceRepository ..|> EvidenceRepository
    SqliteHostAnnotationRepository ..|> HostAnnotationRepository
    SqliteUserAccessRepository ..|> UserAccessRepository

    MemoryUserRepository ..|> EntityCountSource
    SqliteUserRepository ..|> EntityCountSource
    SqliteHostRepository ..|> EntityCountSource
    SqliteEvidenceRepository ..|> EntityCountSource
    SqliteHostAnnotationRepository ..|> EntityCountSource

    SqliteUserRepository --> SqliteDatabase
    SqliteHostRepository --> SqliteDatabase
    SqliteEvidenceRepository --> SqliteDatabase
    SqliteHostAnnotationRepository --> SqliteDatabase
    SqliteUserAccessRepository --> SqliteDatabase

    %% =========================================================
    %% RELACIONAMENTOS DOS ADAPTERS
    %% =========================================================

    PythonLoggingAdapter ..|> LoggerPort
    PythonLoggingAdapter --> PythonLogging : adapta

    NmapAdapter ..|> NetworkScannerPort
    NmapAdapter --> Nmap : adapta

    SshExecutor ..|> RemoteExecutorPort
    SshExecutor --> SSH : adapta

    LinuxLocalNetworkDetector ..|> LocalNetworkDetectorPort

    %% =========================================================
    %% RELACIONAMENTOS DA APRESENTACAO
    %% =========================================================

    CLI --> Container : cria
    CLI --> UserService : cadastro e autenticacao
    CLI --> UserAccessReportService : gera relatorio
    CLI --> HtmlAccessReport : seleciona HTML
    CLI --> PdfAccessReport : seleciona PDF
    CLI --> FacadeSingletonController : acessa casos de uso
    CLI --> CliConfirmacao : confirma operacoes

    TUI --> Container : utiliza
    TUI --> FacadeSingletonController : acessa fachada
    TUI --> TuiConfirmacao : confirma operacoes

    CliConfirmacao ..|> ConfirmationPort
    TuiConfirmacao ..|> ConfirmationPort

    %% =========================================================
    %% NOTAS SOBRE OS PADROES
    %% =========================================================

    note for UserRepository "Repository: define o contrato de persistencia no dominio."
    note for RepositoryFactory "Abstract Factory: cria uma familia de repositories relacionados."
    note for PythonLoggingAdapter "Adapter: converte LoggerPort para logging.Logger."
    note for AccessReportTemplate "Template Method: fixa o algoritmo de geracao do relatorio."
    note for FacadeSingletonController "Facade: oferece um ponto unico de acesso aos gerentes."