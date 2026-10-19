"""Ações (padrão Command). Novas ações = novas subclasses registradas no ActionRegistry (RNF-11)."""
from __future__ import annotations

from abc import ABC, abstractmethod
from typing import ClassVar

from .enums import NivelRisco, TipoAcao, TipoCapacidade
from .models import Host, ResultadoExecucao
from .ports import RemoteExecutorPort
from .risk import classificar_comando, validar_comando_nao_interativo


class Acao(ABC):
    id: ClassVar[str]
    nome: ClassVar[str]
    tipo: ClassVar[TipoAcao]
    capacidades_exigidas: ClassVar[tuple[TipoCapacidade, ...]] = ()
    risco_base: ClassVar[NivelRisco] = NivelRisco.MEDIO

    @classmethod
    def compativel_com(cls, host: Host) -> bool:
        """RN-04: só é compatível se TODAS as capacidades exigidas estiverem disponíveis."""
        return all(host.possui_capacidade(c) for c in cls.capacidades_exigidas)

    def risco(self) -> NivelRisco:
        return self.risco_base

    @abstractmethod
    def descrever(self, host: Host) -> str:
        """Texto legível do que será feito (usado em confirmação, dry-run e auditoria)."""

    @abstractmethod
    def detalhes(self) -> str:
        """Detalhe técnico para o registro de evidência (ex.: o comando exato)."""

    @abstractmethod
    def executar(self, host: Host, remoto: RemoteExecutorPort) -> ResultadoExecucao:
        """Executa a ação de fato."""


class ExecutarComandoAcao(Acao):
    """UC-03 / RF-05: comando remoto não interativo via SSH."""

    id = "executar_comando"
    nome = "Executar comando remoto"
    tipo = TipoAcao.COMANDO_REMOTO
    capacidades_exigidas = (TipoCapacidade.SSH,)
    risco_base = NivelRisco.MEDIO

    def __init__(self, comando: str, timeout_s: float = 30.0) -> None:
        validar_comando_nao_interativo(comando)
        self.comando = comando.strip()
        self.timeout_s = timeout_s

    def risco(self) -> NivelRisco:
        return classificar_comando(self.comando)

    def descrever(self, host: Host) -> str:
        return f"Executar via SSH em {host.ip}: {self.comando}"

    def detalhes(self) -> str:
        return self.comando

    def executar(self, host: Host, remoto: RemoteExecutorPort) -> ResultadoExecucao:
        return remoto.executar(host, self.comando, self.timeout_s)
