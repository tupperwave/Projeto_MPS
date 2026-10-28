"""Portas (interfaces) que a infraestrutura implementa. O domínio/aplicação só conhece estas."""
from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Iterator

from .enums import NivelRisco, TipoConfirmacao
from .annotations import AnotacaoHost
from .models import AlvoRede, Evidencia, FiltroHistorico, Host, ResultadoExecucao
from .users import Usuario


class NetworkScannerPort(ABC):
    @abstractmethod
    def disponivel(self) -> bool: ...

    @abstractmethod
    def varrer(self, alvo: AlvoRede) -> list[Host]:
        """Retorna hosts ativos. Deve levantar ScannerFalhou se a varredura não completou."""


class LocalNetworkDetectorPort(ABC):
    @abstractmethod
    def detectar(self) -> AlvoRede:
        """Retorna o CIDR IPv4 da interface associada à rota padrão."""


class RemoteExecutorPort(ABC):
    @abstractmethod
    def executar(self, host: Host, comando: str, timeout_s: float) -> ResultadoExecucao: ...


class HostRepository(ABC):
    @abstractmethod
    def salvar(self, host: Host) -> Host:
        """Insere ou atualiza (chave: ip). Preenche host.id."""

    @abstractmethod
    def buscar_por_ip(self, ip: str) -> Host | None: ...

    @abstractmethod
    def listar(self, tag: str | None = None) -> list[Host]: ...


class HostAnnotationRepository(ABC):
    @abstractmethod
    def salvar(self, anotacao: AnotacaoHost) -> AnotacaoHost: ...

    @abstractmethod
    def buscar_por_id(self, id_: int) -> AnotacaoHost | None: ...

    @abstractmethod
    def listar(self, host_ip: str | None = None) -> list[AnotacaoHost]: ...

    @abstractmethod
    def excluir(self, id_: int) -> bool: ...


class EvidenceRepository(ABC):
    @abstractmethod
    def salvar(self, evidencia: Evidencia) -> Evidencia: ...

    @abstractmethod
    def ultimo_hash(self) -> str: ...

    @abstractmethod
    def buscar(self, filtro: FiltroHistorico) -> list[Evidencia]: ...

    @abstractmethod
    def buscar_por_id(self, id_: int) -> Evidencia | None: ...

    @abstractmethod
    def todas_em_ordem(self) -> Iterator[Evidencia]: ...


class UserRepository(ABC):
    @abstractmethod
    def salvar(self, usuario: Usuario) -> Usuario: ...

    @abstractmethod
    def buscar_por_login(self, login: str) -> Usuario | None: ...

    @abstractmethod
    def listar(self) -> list[Usuario]: ...


@dataclass(frozen=True)
class PedidoConfirmacao:
    descricao: str
    host_ip: str
    risco: NivelRisco
    tipo: TipoConfirmacao


class ConfirmationPort(ABC):
    """Implementada pela camada de apresentação (CLI/TUI)."""

    @abstractmethod
    def confirmar(self, pedido: PedidoConfirmacao) -> bool: ...
