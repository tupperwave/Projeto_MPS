"""Adapter para o mecanismo logging da biblioteca padrão."""

from __future__ import annotations

import logging

from ..domain.ports import LoggerPort


class PythonLoggingAdapter(LoggerPort):
    """Adapta logging.Logger para a porta LoggerPort."""

    def __init__(
        self,
        logger: logging.Logger,
    ) -> None:
        self._logger = logger

    def info(
        self,
        mensagem: str,
        **dados: object,
    ) -> None:
        contexto = " ".join(
            f"{chave}={valor!r}"
            for chave, valor in sorted(dados.items())
        )
        self._logger.info("%s%s", mensagem, f" {contexto}" if contexto else "")

