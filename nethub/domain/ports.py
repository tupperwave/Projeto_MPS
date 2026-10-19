"""Portas (interfaces) que a infraestrutura implementa. O domínio/aplicação só conhece estas."""
from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Iterator

from .enums import NivelRisco, TipoConfirmacao
from .models import AlvoRede, Evidencia, FiltroHistorico, Host, ResultadoExecucao


class NetworkScannerPort(ABC):
    @abstractmethod
    def disponivel(self) -> bool: ...

    @abstractmethod
    def varrer(self, alvo: AlvoRede) -> list[Host]:
        """Retorna hosts ativos. Deve levantar ScannerFalhou se a varredura não completou."""


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
