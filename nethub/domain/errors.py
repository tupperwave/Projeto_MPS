"""Exceções do domínio. Todas herdam de NetHubError para o CLI/TUI tratarem de forma uniforme."""


class NetHubError(Exception):
    """Erro base do NetHub."""


class AlvoInvalido(NetHubError):
    pass


class NaoAutorizado(NetHubError):
    pass


class ScannerIndisponivel(NetHubError):
    pass


class ScannerFalhou(NetHubError):
    pass


class HostNaoEncontrado(NetHubError):
    pass


class RegistroNaoEncontrado(NetHubError):
    pass


class AcaoIncompativel(NetHubError):
    pass


class DryRunObrigatorio(NetHubError):
    pass


class ComandoInvalido(NetHubError):
    pass


class TagInvalida(NetHubError):
    pass


class LoginInvalido(NetHubError):
    """Login ausente ou fora das regras de cadastro."""


class SenhaInvalida(NetHubError):
    """Senha fora das regras de cadastro."""


class UsuarioDuplicado(NetHubError):
    """Já existe um usuário com o mesmo login."""


class FalhaPersistenciaUsuarios(NetHubError):
    """Falha ao gravar ou consultar usuários no armazenamento permanente."""


class FalhaPersistenciaAcessosRemotos(NetHubError):
    """Falha ao gravar ou consultar acessos remotos no armazenamento permanente."""


class AcessoRemotoDuplicado(NetHubError):
    """Já existe um acesso remoto com os mesmos dados."""

class AcessoRemotoInvalido(NetHubError):
    """Dados inválidos para acesso remoto."""


class FalhaBancoDados(NetHubError):
    """Falha ao abrir ou preparar o banco de dados local."""

