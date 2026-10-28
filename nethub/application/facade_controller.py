"""Fachada única dos gerentes, singleton no escopo de uma composição."""
from __future__ import annotations

from dataclasses import dataclass
from threading import RLock
from typing import TYPE_CHECKING
from weakref import WeakKeyDictionary

from .entity_count_service import EntityCountService

if TYPE_CHECKING:
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
    """Expõe os gerentes e delega a contagem ao serviço correspondente."""

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
            self._inicializada = True

    def quantidade_entidades_cadastradas(self) -> int:
        return self._contador.quantidade_entidades_cadastradas()
