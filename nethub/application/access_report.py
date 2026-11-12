"""Relatórios de acesso usando Template Method."""

from __future__ import annotations

from abc import ABC, abstractmethod

from ..domain.models import EstatisticaAcesso
from ..domain.ports import UserAccessRepository
from io import BytesIO



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


class HtmlAccessReport(AccessReportTemplate):
    def __init__(self) -> None:
        self._partes: list[str] = []

    def iniciar_documento(self) -> None:
        self._partes.append(
            "<!doctype html>"
            "<html lang='pt-BR'>"
            "<head><meta charset='utf-8'>"
            "<title>Relatório de acessos</title></head>"
            "<body>"
            "<table>"
            "<thead><tr>"
            "<th>Login</th>"
            "<th>Total</th>"
            "<th>Sucessos</th>"
            "<th>Falhas</th>"
            "<th>Último acesso</th>"
            "</tr></thead>"
            "<tbody>"
        )

    def escrever_titulo(self, titulo: str) -> None:
        self._partes.append(f"<h1>{titulo}</h1>")

    def escrever_linha(
        self,
        estatistica: EstatisticaAcesso,
    ) -> None:
        ultimo = (
            estatistica.ultimo_acesso.isoformat()
            if estatistica.ultimo_acesso
            else "Nunca"
        )

        self._partes.append(
            "<tr>"
            f"<td>{estatistica.login}</td>"
            f"<td>{estatistica.total}</td>"
            f"<td>{estatistica.sucessos}</td>"
            f"<td>{estatistica.falhas}</td>"
            f"<td>{ultimo}</td>"
            "</tr>"
        )

    def finalizar_documento(self) -> None:
        self._partes.append(
            "</tbody></table></body></html>"
        )

    def resultado(self) -> bytes:
        return "".join(self._partes).encode("utf-8")


class HtmlAccessReport(AccessReportTemplate):
    def __init__(self) -> None:
        self._partes: list[str] = []

    def iniciar_documento(self) -> None:
        self._partes.append(
            "<!doctype html>"
            "<html lang='pt-BR'>"
            "<head><meta charset='utf-8'>"
            "<title>Relatório de acessos</title></head>"
            "<body>"
            "<table>"
            "<thead><tr>"
            "<th>Login</th>"
            "<th>Total</th>"
            "<th>Sucessos</th>"
            "<th>Falhas</th>"
            "<th>Último acesso</th>"
            "</tr></thead>"
            "<tbody>"
        )

    def escrever_titulo(self, titulo: str) -> None:
        self._partes.append(f"<h1>{titulo}</h1>")

    def escrever_linha(
        self,
        estatistica: EstatisticaAcesso,
    ) -> None:
        ultimo = (
            estatistica.ultimo_acesso.isoformat()
            if estatistica.ultimo_acesso
            else "Nunca"
        )

        self._partes.append(
            "<tr>"
            f"<td>{estatistica.login}</td>"
            f"<td>{estatistica.total}</td>"
            f"<td>{estatistica.sucessos}</td>"
            f"<td>{estatistica.falhas}</td>"
            f"<td>{ultimo}</td>"
            "</tr>"
        )

    def finalizar_documento(self) -> None:
        self._partes.append(
            "</tbody></table></body></html>"
        )

    def resultado(self) -> bytes:
        return "".join(self._partes).encode("utf-8")

    

class PdfAccessReport(AccessReportTemplate):
    def __init__(self) -> None:
        self._destino = BytesIO()
        self._pdf = None
        self._y = 0

    def iniciar_documento(self) -> None:
        from reportlab.lib.pagesizes import A4
        from reportlab.pdfgen import canvas

        self._pdf = canvas.Canvas(
            self._destino,
            pagesize=A4,
        )
        self._pdf.setTitle(
            "Relatório de acessos de usuários"
        )
        self._y = 800

    def escrever_titulo(self, titulo: str) -> None:
        self._pdf.drawString(40, self._y, titulo)
        self._y -= 30

    def escrever_linha(
        self,
        estatistica: EstatisticaAcesso,
    ) -> None:
        ultimo = (
            estatistica.ultimo_acesso.isoformat()
            if estatistica.ultimo_acesso
            else "Nunca"
        )

        linha = (
            f"{estatistica.login} | "
            f"total={estatistica.total} | "
            f"sucessos={estatistica.sucessos} | "
            f"falhas={estatistica.falhas} | "
            f"ultimo={ultimo}"
        )

        self._pdf.drawString(40, self._y, linha[:120])
        self._y -= 18

        if self._y < 40:
            self._pdf.showPage()
            self._y = 800

    def finalizar_documento(self) -> None:
        self._pdf.save()

    def resultado(self) -> bytes:
        return self._destino.getvalue()








class UserAccessReportService:
    def __init__(
        self,
        repositorio: UserAccessRepository,
    ) -> None:
        self._repositorio = repositorio

    def gerar(
        self,
        formato: AccessReportTemplate,
    ) -> bytes:
        estatisticas = self._repositorio.estatisticas()
        return formato.gerar(estatisticas)