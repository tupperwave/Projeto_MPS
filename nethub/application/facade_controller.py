"""Fachada única dos gerentes, singleton no escopo de uma composição."""
from __future__ import annotations

from dataclasses import dataclass
from threading import RLock
from typing import TYPE_CHECKING
from weakref import WeakKeyDictionary

from .entity_count_service import EntityCountService
from .commands import (
    CommandInvoker,
    CreateAnnotationCommand,
    DeleteAnnotationCommand,
    GetAnnotationCommand,
    ListAnnotationsCommand,
    UndoLastAnnotationUpdateCommand,
    UpdateAnnotationCommand,
)

if TYPE_CHECKING:
    from ..domain.annotations import AnotacaoHost
    from .action_executor import ActionExecutor
    from .action_registry import ContextMenuService
    from .annotation_service import HostAnnotationService
    from .discovery_service import DiscoveryService
    from .evidence_service import EvidenceService, HistoryService
    from .inventory_service import InventoryService
    from .user_service import UserService


class FacadeSingletonScope:
    """Identidade de uma composição da aplicação; usada como chave fraca."""


@dataclass(frozen=True)
class GerentesController:
    inventario: InventoryService
    descoberta: DiscoveryService
    menu: ContextMenuService
    executor: ActionExecutor
    evidencias: EvidenceService
    historico: HistoryService
    usuarios: UserService
    anotacoes: HostAnnotationService


class FacadeSingletonController:
    """Fachada dos casos de uso, única dentro de cada composição da aplicação."""

    _instances: WeakKeyDictionary[FacadeSingletonScope, "FacadeSingletonController"] = WeakKeyDictionary()
    _lock = RLock()

    def __new__(
        cls,
        escopo: FacadeSingletonScope,
        gerentes: GerentesController,
        contador: EntityCountService,
    ) -> "FacadeSingletonController":
        with cls._lock:
            instancia = cls._instances.get(escopo)
            if instancia is None:
                instancia = super().__new__(cls)
                cls._instances[escopo] = instancia
            return instancia

    def __init__(
        self,
        escopo: FacadeSingletonScope,
        gerentes: GerentesController,
        contador: EntityCountService,
    ) -> None:
        del escopo  # Chave fraca: a fachada não prolonga o ciclo de vida do escopo.
        with self._lock:
            if getattr(self, "_inicializada", False):
                return
            self.gerentes = gerentes
            self._contador = contador
            self._command_invoker = CommandInvoker()
            self._inicializada = True

    def quantidade_entidades_cadastradas(self) -> int:
        return self._contador.quantidade_entidades_cadastradas()

    def criar_anotacao(self, host_ip: str, conteudo: str) -> AnotacaoHost:
        return self._command_invoker.invoke(
            CreateAnnotationCommand(self.gerentes.anotacoes, host_ip, conteudo)
        )

    def obter_anotacao(self, id_: int) -> AnotacaoHost:
        return self._command_invoker.invoke(
            GetAnnotationCommand(self.gerentes.anotacoes, id_)
        )

    def listar_anotacoes(self, host_ip: str | None = None) -> list[AnotacaoHost]:
        return self._command_invoker.invoke(
            ListAnnotationsCommand(self.gerentes.anotacoes, host_ip)
        )

    def atualizar_anotacao(self, id_: int, conteudo: str) -> AnotacaoHost:
        return self._command_invoker.invoke(
            UpdateAnnotationCommand(self.gerentes.anotacoes, id_, conteudo)
        )

    def excluir_anotacao(self, id_: int) -> bool:
        return self._command_invoker.invoke(
            DeleteAnnotationCommand(self.gerentes.anotacoes, id_)
        )

    def desfazer_ultima_atualizacao_anotacao(self) -> AnotacaoHost:
        return self._command_invoker.invoke(
            UndoLastAnnotationUpdateCommand(self.gerentes.anotacoes)
        )
