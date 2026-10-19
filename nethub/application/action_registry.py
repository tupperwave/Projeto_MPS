"""Registro de ações e menu contextual (RF-03, RN-04, RNF-11)."""
from __future__ import annotations

from ..domain.actions import Acao, ExecutarComandoAcao
from ..domain.enums import StatusCapacidade, TipoCapacidade
from ..domain.models import Host


class ActionRegistry:
    def __init__(self) -> None:
        self._acoes: dict[str, type[Acao]] = {}

    def registrar(self, acao: type[Acao]) -> None:
        self._acoes[acao.id] = acao

    def obter(self, id_: str) -> type[Acao]:
        return self._acoes[id_]

    def todas(self) -> list[type[Acao]]:
        return list(self._acoes.values())


def registro_padrao() -> ActionRegistry:
    registro = ActionRegistry()
    registro.registrar(ExecutarComandoAcao)
    return registro


class ContextMenuService:
    def __init__(self, registro: ActionRegistry) -> None:
        self._registro = registro

    def acoes_compativeis(self, host: Host) -> list[type[Acao]]:
        return [a for a in self._registro.todas() if a.compativel_com(host)]

    def capacidades_nao_verificadas(self, host: Host) -> list[TipoCapacidade]:
        exigidas = {c for a in self._registro.todas() for c in a.capacidades_exigidas}
        return sorted(
            (c for c in exigidas if host.status_capacidade(c) is StatusCapacidade.DESCONHECIDA),
            key=lambda c: c.value,
        )
