"""Fábrica de repositórios da infraestrutura."""

from __future__ import annotations

from dataclasses import dataclass

from ..domain.ports import (
    EvidenceRepository,
    HostAnnotationRepository,
    HostRepository,
    UserRepository,
    UserAccessRepository,
)
from .memory_user_repository import MemoryUserRepository
from .sqlite_db import SqliteDatabase
from .sqlite_repositories import (
    SqliteEvidenceRepository,
    SqliteHostAnnotationRepository,
    SqliteHostRepository,
    SqliteUserRepository,
    SqliteUserAccessRepository,
)

@dataclass(frozen=True)
class Repositories:
    """Família de repositórios usada pela aplicação."""

    hosts: HostRepository
    evidencias: EvidenceRepository
    usuarios: UserRepository
    anotacoes: HostAnnotationRepository


class RepositoryFactory:
    """Abstract Factory concreta dos repositórios do sistema."""

    def __init__(
        self,
        banco: SqliteDatabase,
        persistencia_usuarios: str = "memoria",
    ) -> None:
        self._banco = banco
        self._persistencia_usuarios = persistencia_usuarios

    def criar(self) -> Repositories:
        return Repositories(
            hosts=SqliteHostRepository(self._banco),
            evidencias=SqliteEvidenceRepository(self._banco),
            usuarios=self._criar_usuarios(),
            anotacoes=SqliteHostAnnotationRepository(self._banco),
            acessos=SqliteUserAccessRepository(self._banco),
        )

    def _criar_usuarios(self) -> UserRepository:
        if self._persistencia_usuarios == "memoria":
            return MemoryUserRepository()

        if self._persistencia_usuarios == "sqlite":
            return SqliteUserRepository(self._banco)

        raise ValueError(
            "Persistência de usuários deve ser 'memoria' ou 'sqlite'."
        )

