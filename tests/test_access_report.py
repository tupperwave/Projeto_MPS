"""Testes dos formatos de relatório de acesso e do serviço de aplicação."""
from __future__ import annotations

import unittest
from types import SimpleNamespace
from datetime import datetime, timezone
from io import BytesIO
from unittest.mock import patch

from nethub.application.access_report import (
    HtmlAccessReport,
    PdfAccessReport,
    PlainTextAccessReport,
    UserAccessReportService,
)
from nethub.domain.models import EstatisticaAcesso


class FakeUserAccessRepository:
    def __init__(self, estatisticas: list[EstatisticaAcesso]) -> None:
        self._estatisticas = estatisticas
        self.chamadas = 0

    def estatisticas(self) -> list[EstatisticaAcesso]:
        self.chamadas += 1
        return self._estatisticas


class TestAccessReports(unittest.TestCase):
    def setUp(self) -> None:
        self.estatistica = EstatisticaAcesso(
            login="Ana",
            total=3,
            sucessos=2,
            falhas=1,
            ultimo_acesso=datetime(2026, 11, 25, 12, tzinfo=timezone.utc),
        )

    def test_relatorio_texto_inclui_titulo_e_estatistica(self) -> None:
        conteudo = PlainTextAccessReport().gerar([self.estatistica]).decode("utf-8")

        self.assertIn("Relatório de acessos de usuários", conteudo)
        self.assertIn("login='Ana'", conteudo)
        self.assertIn("total=3", conteudo)

    def test_relatorio_html_e_reutilizavel_sem_duplicar_documentos(self) -> None:
        formato = HtmlAccessReport()
        primeiro = formato.gerar([self.estatistica])
        segundo = formato.gerar([self.estatistica])

        self.assertEqual(primeiro, segundo)
        texto = segundo.decode("utf-8")
        self.assertEqual(texto.count("<html"), 1)
        self.assertIn("<td>Ana</td>", texto)

    def test_relatorio_pdf_tem_assinatura_pdf(self) -> None:
        class FakePdf:
            def __init__(self, destino: BytesIO) -> None:
                self.destino = destino

            def setTitle(self, _title: str) -> None:
                pass

            def drawString(self, _x: int, _y: int, _text: str) -> None:
                pass

            def showPage(self) -> None:
                pass

            def save(self) -> None:
                self.destino.write(b"%PDF-fake")

        def importar(nome: str) -> SimpleNamespace:
            if nome == "reportlab.lib.pagesizes":
                return SimpleNamespace(A4=(595, 842))
            if nome == "reportlab.pdfgen.canvas":
                return SimpleNamespace(
                    Canvas=lambda destino, *_args, **_kwargs: FakePdf(destino)
                )
            raise AssertionError(f"Import inesperado: {nome}")

        with patch("nethub.application.access_report.import_module", side_effect=importar):
            conteudo = PdfAccessReport().gerar([self.estatistica])

        self.assertTrue(conteudo.startswith(b"%PDF"))

    def test_servico_consulta_repositorio_e_gera_formato_solicitado(self) -> None:
        repositorio = FakeUserAccessRepository([self.estatistica])
        servico = UserAccessReportService(repositorio)

        conteudo = servico.gerar(PlainTextAccessReport()).decode("utf-8")

        self.assertEqual(repositorio.chamadas, 1)
        self.assertIn("login='Ana'", conteudo)


if __name__ == "__main__":
    unittest.main()
