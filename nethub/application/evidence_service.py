"""Registro e consulta de evidências (RF-09, RF-14, RN-03, RNF-06)."""
from __future__ import annotations

from ..domain.errors import RegistroNaoEncontrado
from ..domain.models import Evidencia, FiltroHistorico
from ..domain.ports import EvidenceRepository


class EvidenceService:
    def __init__(self, repositorio: EvidenceRepository) -> None:
        self._repo = repositorio

    def registrar(self, evidencia: Evidencia) -> Evidencia:
        """Sela a evidência encadeando-a ao hash da anterior e a persiste."""
        evidencia.selar(self._repo.ultimo_hash())
        return self._repo.salvar(evidencia)

    def verificar_cadeia(self) -> tuple[bool, int | None]:
        """Retorna (ok, id_do_primeiro_registro_inconsistente)."""
        anterior = ""
        for ev in self._repo.todas_em_ordem():
            if ev.hash_anterior != anterior or not ev.integra():
                return False, ev.id
            anterior = ev.hash_integridade
        return True, None


class HistoryService:
    """UC-08: consulta somente leitura."""

    def __init__(self, repositorio: EvidenceRepository) -> None:
        self._repo = repositorio

    def consultar(self, filtro: FiltroHistorico | None = None) -> list[Evidencia]:
        return self._repo.buscar(filtro or FiltroHistorico())

    def detalhar(self, id_: int) -> Evidencia:
        ev = self._repo.buscar_por_id(id_)
        if ev is None:
            raise RegistroNaoEncontrado(f"Registro #{id_} não encontrado.")
        return ev
