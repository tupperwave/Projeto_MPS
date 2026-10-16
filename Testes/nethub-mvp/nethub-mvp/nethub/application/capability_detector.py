"""Detecção de capacidades. MVP: apenas passiva (a partir das portas vistas pelo Nmap).

SSH 'disponivel' significa porta SSH aberta (alcançável), NÃO que a autenticação funciona.
As demais capacidades permanecem 'desconhecida' até existir sondagem ativa (próxima iteração).
"""
from __future__ import annotations

from ..domain.enums import StatusCapacidade, TipoCapacidade
from ..domain.models import Host, agora


class CapabilityDetector:
    def detectar_passivo(self, host: Host) -> None:
        quando = agora()
        abertas = host.portas_abertas()
        porta = next((p for p in abertas if p.servico == "ssh"), None) or next(
            (p for p in abertas if p.numero == 22), None
        )
        if porta is not None:
            host.porta_ssh = porta.numero
            host.definir_capacidade(TipoCapacidade.SSH, StatusCapacidade.DISPONIVEL, quando)
        else:
            host.definir_capacidade(TipoCapacidade.SSH, StatusCapacidade.DESCONHECIDA, quando)
