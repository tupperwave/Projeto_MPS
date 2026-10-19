"""Inventário de hosts (RF-02, RF-15)."""
from __future__ import annotations

from ..domain.errors import HostNaoEncontrado
from ..domain.models import Host, agora, normalizar_tag
from ..domain.ports import HostRepository
from .capability_detector import CapabilityDetector


class InventoryService:
    def __init__(self, repositorio: HostRepository, detector: CapabilityDetector) -> None:
        self._repo = repositorio
        self._detector = detector

    def listar(self, tag: str | None = None) -> list[Host]:
        return self._repo.listar(normalizar_tag(tag) if tag else None)

    def obter(self, ip: str) -> Host:
        host = self._repo.buscar_por_ip(ip)
        if host is None:
            raise HostNaoEncontrado(f"Host {ip} não está no inventário. Rode a descoberta primeiro.")
        return host

    def registrar_descoberta(self, encontrado: Host) -> tuple[Host, bool]:
        """Insere ou atualiza um host visto numa varredura. Retorna (host, criado)."""
        quando = agora()
        existente = self._repo.buscar_por_ip(encontrado.ip)
        if existente is None:
            encontrado.primeira_descoberta = encontrado.ultima_descoberta = quando
            host, criado = encontrado, True
        else:
            existente.mesclar_descoberta(encontrado, quando)
            host, criado = existente, False
        self._detector.detectar_passivo(host)
        return self._repo.salvar(host), criado

    def adicionar_tag(self, ip: str, tag: str) -> Host:
        host = self.obter(ip)
        host.adicionar_tag(tag)
        return self._repo.salvar(host)

    def remover_tag(self, ip: str, tag: str) -> Host:
        host = self.obter(ip)
        host.remover_tag(tag)
        return self._repo.salvar(host)
