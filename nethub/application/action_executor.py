"""Orquestra a execução de uma ação: compatibilidade -> risco -> dry-run -> confirmação -> execução -> evidência."""
from __future__ import annotations

from dataclasses import dataclass

from ..domain.actions import Acao
from ..domain.enums import NivelRisco, StatusExecucao, TipoConfirmacao
from ..domain.errors import AcaoIncompativel, DryRunObrigatorio
from ..domain.models import Evidencia, Host, ResultadoExecucao, truncar
from ..domain.ports import ConfirmationPort, PedidoConfirmacao, RemoteExecutorPort
from .evidence_service import EvidenceService
from .risk_policy import RiskPolicy


@dataclass(frozen=True)
class PlanoSimulado:
    """Resultado do dry-run: o que SERIA feito, sem executar nada."""

    descricao: str
    host_ip: str
    detalhes: str
    risco: NivelRisco
    tipo_confirmacao: TipoConfirmacao
    exige_dry_run: bool


@dataclass
class ResultadoAcao:
    resultado: ResultadoExecucao
    evidencia: Evidencia


class ActionExecutor:
    def __init__(
        self, remoto: RemoteExecutorPort, evidencias: EvidenceService, politica: RiskPolicy | None = None
    ) -> None:
        self._remoto = remoto
        self._evidencias = evidencias
        self._politica = politica or RiskPolicy()

    def simular(self, acao: Acao, host: Host) -> PlanoSimulado:
        self._garantir_compatibilidade(acao, host)
        risco = acao.risco()
        return PlanoSimulado(
            descricao=acao.descrever(host),
            host_ip=host.ip,
            detalhes=acao.detalhes(),
            risco=risco,
            tipo_confirmacao=self._politica.tipo_confirmacao(risco),
            exige_dry_run=self._politica.exige_dry_run(risco),
        )

    def executar(
        self,
        acao: Acao,
        host: Host,
        confirmacao: ConfirmationPort,
        dry_run_realizado: bool = False,
    ) -> ResultadoAcao:
        self._garantir_compatibilidade(acao, host)
        risco = acao.risco()
        if self._politica.exige_dry_run(risco) and not dry_run_realizado:
            raise DryRunObrigatorio(
                f"Ação de risco {risco.rotulo} exige dry-run (simulação apresentada ao operador) antes da execução."
            )
        descricao = acao.descrever(host)
        tipo = self._politica.tipo_confirmacao(risco)
        if tipo is not TipoConfirmacao.NENHUMA:
            if not confirmacao.confirmar(PedidoConfirmacao(descricao, host.ip, risco, tipo)):
                return self._registrar(acao, host, descricao, risco, ResultadoExecucao(StatusExecucao.CANCELADA))
        try:
            resultado = acao.executar(host, self._remoto)
        except Exception as exc:  # falha inesperada também vira evidência
            resultado = ResultadoExecucao(StatusExecucao.FALHA, stderr=f"Erro inesperado: {exc}")
        return self._registrar(acao, host, descricao, risco, resultado)

    # ------------------------------------------------------------------ internos
    def _garantir_compatibilidade(self, acao: Acao, host: Host) -> None:
        if not type(acao).compativel_com(host):
            exigidas = ", ".join(c.value for c in type(acao).capacidades_exigidas)
            raise AcaoIncompativel(f"'{acao.nome}' não é compatível com {host.ip} (capacidades exigidas: {exigidas}).")

    def _registrar(
        self, acao: Acao, host: Host, descricao: str, risco: NivelRisco, resultado: ResultadoExecucao
    ) -> ResultadoAcao:
        evidencia = self._evidencias.registrar(
            Evidencia(
                tipo_acao=acao.tipo,
                descricao=descricao,
                alvo=host.ip,
                risco=risco,
                status=resultado.status,
                detalhes=acao.detalhes(),
                saida=truncar(resultado.stdout),
                erro=truncar(resultado.stderr),
                exit_code=resultado.exit_code,
            )
        )
        return ResultadoAcao(resultado, evidencia)
