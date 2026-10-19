"""Política de risco: define confirmação e dry-run proporcionais (RF-04, RN-01, RN-05)."""
from __future__ import annotations

from ..domain.enums import NivelRisco, TipoConfirmacao


class RiskPolicy:
    _CONFIRMACAO = {
        NivelRisco.BAIXO: TipoConfirmacao.NENHUMA,
        NivelRisco.MEDIO: TipoConfirmacao.SIMPLES,
        NivelRisco.ALTO: TipoConfirmacao.EXPLICITA,
        NivelRisco.CRITICO: TipoConfirmacao.EXPLICITA,
    }

    def tipo_confirmacao(self, risco: NivelRisco) -> TipoConfirmacao:
        return self._CONFIRMACAO[risco]

    def exige_dry_run(self, risco: NivelRisco) -> bool:
        return risco >= NivelRisco.CRITICO
