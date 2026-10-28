"""Raiz de composição: monta o grafo de dependências (injeção manual)."""
from __future__ import annotations

import os
import sqlite3
from dataclasses import dataclass, field
from pathlib import Path
from weakref import finalize

from .application.action_executor import ActionExecutor
from .application.action_registry import ActionRegistry, ContextMenuService, registro_padrao
from .application.capability_detector import CapabilityDetector
from .application.discovery_service import DiscoveryService
from .application.evidence_service import EvidenceService, HistoryService
from .application.inventory_service import InventoryService
from .application.annotation_service import HostAnnotationService
from .application.entity_count_service import EntityCountService
from .application.facade_controller import (
    FacadeSingletonController, FacadeSingletonScope, GerentesController,
)
from .application.user_service import UserService
from .domain.errors import FalhaBancoDados
from .domain.ports import LocalNetworkDetectorPort, NetworkScannerPort, RemoteExecutorPort
from .infrastructure.local_network_detector import LinuxLocalNetworkDetector
from .infrastructure.nmap_adapter import NmapAdapter
from .infrastructure.memory_user_repository import MemoryUserRepository
from .infrastructure.sqlite_db import SqliteDatabase
from .infrastructure.sqlite_repositories import (
    SqliteEvidenceRepository, SqliteHostAnnotationRepository, SqliteHostRepository, SqliteUserRepository,
)
from .infrastructure.ssh_adapter import SshExecutor


@dataclass
class Container:
    banco: SqliteDatabase
    registro: ActionRegistry
    inventario: InventoryService
    descoberta: DiscoveryService
    menu: ContextMenuService
    executor: ActionExecutor
    evidencias: EvidenceService
    historico: HistoryService
    usuarios: UserService
    anotacoes: HostAnnotationService
    gerentes: GerentesController
    contador_entidades: EntityCountService
    facade_scope: FacadeSingletonScope
    facade: FacadeSingletonController
    rede_local: LocalNetworkDetectorPort
    _finalizador: finalize = field(init=False, repr=False, compare=False)

    def __post_init__(self) -> None:
        self._finalizador = finalize(self, self.banco.fechar)

    def fechar(self) -> None:
        """Fecha os recursos que pertencem a esta composição."""
        self._finalizador()


def caminho_padrao_db() -> Path:
    return Path(os.environ.get("NETHUB_DB") or Path.home() / ".nethub" / "nethub.db")


def criar_container(
    caminho_db: str | Path | None = None,
    usuario: str | None = None,
    identidade: str | None = None,
    scanner: NetworkScannerPort | None = None,
    remoto: RemoteExecutorPort | None = None,
    persistencia_usuarios: str = "memoria",
) -> Container:
    try:
        banco = SqliteDatabase(caminho_db or caminho_padrao_db())
    except (OSError, sqlite3.Error) as exc:
        raise FalhaBancoDados("Não foi possível abrir ou inicializar o banco SQLite.") from exc
    repo_hosts = SqliteHostRepository(banco)
    repo_evid = SqliteEvidenceRepository(banco)
    evidencias = EvidenceService(repo_evid)
    inventario = InventoryService(repo_hosts, CapabilityDetector())
    registro = registro_padrao()
    if persistencia_usuarios == "memoria":
        repo_usuarios = MemoryUserRepository()
    elif persistencia_usuarios == "sqlite":
        repo_usuarios = SqliteUserRepository(banco)
    else:
        raise ValueError("Persistência de usuários deve ser 'memoria' ou 'sqlite'.")
    usuarios = UserService(repo_usuarios)
    repo_anotacoes = SqliteHostAnnotationRepository(banco)
    anotacoes = HostAnnotationService(repo_anotacoes, repo_hosts)
    descoberta = DiscoveryService(scanner or NmapAdapter(), inventario, evidencias)
    menu = ContextMenuService(registro)
    executor = ActionExecutor(remoto or SshExecutor(usuario=usuario, identidade=identidade), evidencias)
    historico = HistoryService(repo_evid)
    gerentes = GerentesController(inventario, descoberta, menu, executor, evidencias, historico, usuarios, anotacoes)
    contador_entidades = EntityCountService((repo_hosts, repo_usuarios, repo_anotacoes))
    facade_scope = FacadeSingletonScope()
    facade = FacadeSingletonController(facade_scope, gerentes, contador_entidades)
    rede_local = LinuxLocalNetworkDetector()
    return Container(
        banco=banco,
        registro=registro,
        inventario=inventario,
        descoberta=descoberta,
        menu=menu,
        executor=executor,
        evidencias=evidencias,
        historico=historico,
        usuarios=usuarios,
        anotacoes=anotacoes,
        gerentes=gerentes,
        contador_entidades=contador_entidades,
        facade_scope=facade_scope,
        facade=facade,
        rede_local=rede_local,
    )
