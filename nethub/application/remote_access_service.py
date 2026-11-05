"""Serviço de aplicação para gerenciar acessos remotos."""

from dataclasses import dataclass, field
from domain.enums import MetodoAutenticacao
from ..domain.ports import RemoteAccessRepository, SecretStorePort
from domain.access import AcessoRemoto

@dataclass(frozen=True)
class ContextoAcesso:
    host_ip: str
    usuario: str
    porta: int
    metodo: MetodoAutenticacao
    identidade: str | None = None
    senha: str | None = field(default=None, repr=False)


class RemoteAccessService:
    def __init__(
        self,
        repositorio: RemoteAccessRepository,
        armazenamento_segredos: SecretStorePort,
    ) -> None:
        self._repositorio = repositorio
        self._armazenamento_segredos = armazenamento_segredos

    def cadastrar(
        self,
        acesso: AcessoRemoto,
        segredo: str | None = None,
    ) -> AcessoRemoto:
        """Persiste um acesso já validado e, se informado, sua senha."""
        self._repositorio.salvar(acesso)

        if segredo is not None:
            self._armazenamento_segredos.salvar(acesso.host_ip, segredo)

        return acesso

    def obter(self, host_ip: str) -> AcessoRemoto | None:
        return self._repositorio.obter(host_ip)

    def remover(self, host_ip: str) -> bool:
        acesso = self.obter(host_ip)
        if acesso is None:
            return False

        self._repositorio.remover(host_ip)
        self._armazenamento_segredos.remover(host_ip)
        return True

    def resolver_credencial(self, host_ip: str) -> ContextoAcesso:
        acesso = self.obter(host_ip)
        if acesso is None:
            raise LookupError(f"Acesso remoto não encontrado: {host_ip}")

        senha = self._armazenamento_segredos.obter(host_ip)
        return ContextoAcesso(
            host_ip=acesso.host_ip,
            usuario=acesso.usuario,
            porta=acesso.porta,
            metodo=acesso.metodo,
            identidade=acesso.identidade,
            senha=senha,
        )