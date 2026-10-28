"""Contagem agregada de entidades por provedores independentes de CRUD."""
from __future__ import annotations

from collections.abc import Iterable
from typing import Protocol


class EntityCountSource(Protocol):
    """Capacidade opcional de contagem, segregada dos contratos CRUD."""

    def contar(self) -> int: ...


class EntityCountService:
    def __init__(self, fontes: Iterable[EntityCountSource]) -> None:
        self._fontes = tuple(fontes)

    def quantidade_entidades_cadastradas(self) -> int:
        return sum(fonte.contar() for fonte in self._fontes)
