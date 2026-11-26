"""Repositórios em memória para a família de persistência volátil."""
from __future__ import annotations

from ..domain.annotations import AnotacaoHost
from ..domain.models import Evidencia, FiltroHistorico, Host
from ..domain.ports import EvidenceRepository, HostAnnotationRepository, HostRepository
from ..domain.users import Usuario
from .memory_user_repository import MemoryUserRepository


class MemoryHostRepository(HostRepository):
    def __init__(self) -> None:
        self._items: dict[str, Host] = {}
        self._next_id = 1

    def salvar(self, host: Host) -> Host:
        if host.id is None:
            host.id = self._next_id
            self._next_id += 1
        self._items[host.ip] = host
        return host

    def buscar_por_ip(self, ip: str) -> Host | None:
        return self._items.get(ip)

    def listar(self, tag: str | None = None) -> list[Host]:
        items = [h for h in self._items.values() if tag is None or tag in h.tags]
        return sorted(items, key=lambda h: h.ip)

    def contar(self) -> int:
        return len(self._items)


class MemoryHostAnnotationRepository(HostAnnotationRepository):
    def __init__(self) -> None:
        self._items: dict[int, AnotacaoHost] = {}
        self._next_id = 1

    def salvar(self, anotacao: AnotacaoHost) -> AnotacaoHost:
        if anotacao.id is None:
            anotacao.id = self._next_id
            self._next_id += 1
        self._items[anotacao.id] = anotacao
        return anotacao

    def buscar_por_id(self, id_: int) -> AnotacaoHost | None:
        return self._items.get(id_)

    def listar(self, host_ip: str | None = None) -> list[AnotacaoHost]:
        return [a for a in self._items.values() if host_ip is None or a.host_ip == host_ip]

    def excluir(self, id_: int) -> bool:
        return self._items.pop(id_, None) is not None

    def contar(self) -> int:
        return len(self._items)


class MemoryEvidenceRepository(EvidenceRepository):
    def __init__(self) -> None:
        self._items: list[Evidencia] = []

    def salvar(self, evidencia: Evidencia) -> Evidencia:
        evidencia.id = len(self._items) + 1
        self._items.append(evidencia)
        return evidencia

    def ultimo_hash(self) -> str:
        return self._items[-1].hash_integridade if self._items else ""

    def buscar(self, filtro: FiltroHistorico) -> list[Evidencia]:
        items = [e for e in self._items if
            (not filtro.alvo or e.alvo == filtro.alvo) and
            (not filtro.tipo_acao or e.tipo_acao == filtro.tipo_acao) and
            (not filtro.status or e.status == filtro.status) and
            (not filtro.de or e.timestamp >= filtro.de) and
            (not filtro.ate or e.timestamp <= filtro.ate)]
        return list(reversed(items))[:max(1, filtro.limite)]

    def buscar_por_id(self, id_: int) -> Evidencia | None:
        return next((e for e in self._items if e.id == id_), None)

    def todas_em_ordem(self):
        return iter(self._items)

    def contar(self) -> int:
        return len(self._items)
