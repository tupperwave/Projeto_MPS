"""UC-01 / RF-01: descoberta de rede com autorização explícita (RNF-10) e auditoria (RN-03)."""
from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime

from ..domain.enums import NivelRisco, StatusExecucao, TipoAcao
from ..domain.errors import NaoAutorizado, NetHubError, ScannerIndisponivel
from ..domain.models import AlvoRede, Evidencia, Host, agora
from ..domain.ports import NetworkScannerPort
from .evidence_service import EvidenceService
from .inventory_service import InventoryService


@dataclass
class ResultadoDescoberta:
    alvo: AlvoRede
    hosts: list[Host] = field(default_factory=list)
    novos: int = 0
    atualizados: int = 0
    iniciada_em: datetime | None = None
    concluida_em: datetime | None = None


class DiscoveryService:
    def __init__(
        self, scanner: NetworkScannerPort, inventario: InventoryService, evidencias: EvidenceService
    ) -> None:
        self._scanner = scanner
        self._inventario = inventario
        self._evidencias = evidencias

    def descobrir(self, alvo: AlvoRede, autorizado: bool) -> ResultadoDescoberta:
        if not autorizado:
            raise NaoAutorizado("Varredura bloqueada: é necessário confirmar que você tem autorização para varrer este alvo.")
        inicio = agora()
        try:
            if not self._scanner.disponivel():
                raise ScannerIndisponivel(
                    "Nmap não encontrado no PATH. No Fedora, instale com 'sudo dnf install nmap' "
                    "e tente novamente."
                )
            encontrados = self._scanner.varrer(alvo)
        except NetHubError as exc:
            self._evidencias.registrar(
                Evidencia(
                    tipo_acao=TipoAcao.DESCOBERTA,
                    descricao=f"Descoberta de rede em {alvo.cidr}",
                    alvo=alvo.cidr,
                    risco=NivelRisco.MEDIO,
                    status=StatusExecucao.FALHA,
                    detalhes=alvo.cidr,
                    erro=str(exc),
                )
            )
            raise

        resultado = ResultadoDescoberta(alvo=alvo, iniciada_em=inicio)
        for achado in encontrados:
            host, criado = self._inventario.registrar_descoberta(achado)
            resultado.hosts.append(host)
            if criado:
                resultado.novos += 1
            else:
                resultado.atualizados += 1
        resultado.concluida_em = agora()

        resumo = (
            f"{len(resultado.hosts)} host(s): " + ", ".join(h.ip for h in resultado.hosts)
            if resultado.hosts
            else "Varredura concluída sem hosts ativos."
        )
        self._evidencias.registrar(
            Evidencia(
                tipo_acao=TipoAcao.DESCOBERTA,
                descricao=f"Descoberta de rede em {alvo.cidr}",
                alvo=alvo.cidr,
                risco=NivelRisco.MEDIO,
                status=StatusExecucao.CONCLUIDA,
                detalhes=alvo.cidr,
                saida=resumo,
            )
        )
        return resultado
