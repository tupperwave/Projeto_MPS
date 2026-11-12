"""Relatórios de acesso usando Template Method."""

from __future__ import annotations

from abc import ABC, abstractmethod

from ..domain.models import EstatisticaAcesso


class AccessReportTemplate(ABC):
    """Define o algoritmo fixo de geração do relatório."""

    def gerar(
        self,
        estatisticas: list[EstatisticaAcesso],
    ) -> bytes:
        self.iniciar_documento()
        self.escrever_titulo(
            "Relatório de acessos de usuários"
        )

        for estatistica in estatisticas:
            self.escrever_linha(estatistica)

        self.finalizar_documento()
        return self.resultado()

    @abstractmethod
    def iniciar_documento(self) -> None:
        ...

    @abstractmethod
    def escrever_titulo(self, titulo: str) -> None:
        ...

    @abstractmethod
    def escrever_linha(
        self,
        estatistica: EstatisticaAcesso,
    ) -> None:
        ...

    @abstractmethod
    def finalizar_documento(self) -> None:
        ...

    @abstractmethod
    def resultado(self) -> bytes:
        ...


class PlainTextAccessReport(AccessReportTemplate):
    """Gera o relatório em texto simples codificado em UTF-8."""

    def __init__(self) -> None:
        self._linhas: list[str] = []

    def iniciar_documento(self) -> None:
        self._linhas = []

    def escrever_titulo(self, titulo: str) -> None:
        self._linhas.append(titulo)

    def escrever_linha(self, estatistica: EstatisticaAcesso) -> None:
        self._linhas.append(str(estatistica))

    def finalizar_documento(self) -> None:
        # Mantém o documento pronto para ser convertido em bytes sem
        # adicionar espaços ou linhas vazias ao conteúdo gerado.
        pass

    def resultado(self) -> bytes:
        return "\n".join(self._linhas).encode("utf-8")