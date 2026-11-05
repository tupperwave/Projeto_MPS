"""Serviço de aplicação para gerenciar acessos remotos."""

from dataclasses import dataclass, field
from ..domain.enums import MetodoAutenticacao
from ..domain.errors import FalhaPersistenciaAcessosRemotos
from ..domain.ports import RemoteAccessRepository, SecretStorePort
from ..domain.access import AcessoRemoto

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
        referencia = acesso.referencia_segredo or acesso.host_ip
        try:
            if segredo is not None:
                self._armazenamento_segredos.salvar(referencia, segredo)
            return self._repositorio.salvar(acesso)
        except FalhaPersistenciaAcessosRemotos:
            if segredo is not None:
                self._armazenamento_segredos.excluir(referencia)
            raise

    def obter(self, host_ip: str) -> AcessoRemoto | None:
        return self._repositorio.buscar_por_host(host_ip)

    def remover(self, host_ip: str) -> bool:
        acesso = self.obter(host_ip)
        if acesso is None:
            return False
        self._repositorio.excluir_por_host(host_ip)
        if acesso.referencia_segredo:
            self._armazenamento_segredos.excluir(acesso.referencia_segredo)
        return True

    def resolver_credencial(self, host_ip: str) -> ContextoAcesso:
        acesso = self.obter(host_ip)
        if acesso is None:
            raise LookupError(f"Acesso remoto não encontrado: {host_ip}")

        referencia = acesso.referencia_segredo or acesso.host_ip
        senha = self._armazenamento_segredos.obter(referencia)
        return ContextoAcesso(
            host_ip=acesso.host_ip,
            usuario=acesso.usuario,
            porta=acesso.porta,
            metodo=acesso.metodo,
            identidade=acesso.identidade,
            senha=senha,
        )