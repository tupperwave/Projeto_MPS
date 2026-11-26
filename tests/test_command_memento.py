"""Testes do padrão Command e do desfazer da última atualização de anotação."""
from __future__ import annotations

import tempfile
import unittest
from argparse import Namespace
from contextlib import redirect_stdout
from io import StringIO
from pathlib import Path

from nethub.bootstrap import criar_container
from nethub.domain.errors import SemAtualizacaoParaDesfazer
from nethub.domain.models import Host
from nethub.presentation.cli import CliAnotacoes


class TestCommandMementoAnotacao(unittest.TestCase):
    def setUp(self) -> None:
        self.container = criar_container(":memory:")
        self.container.inventario.registrar_descoberta(Host("10.0.0.10"))
        self.anotacao = self.container.facade.criar_anotacao(
            "10.0.0.10", "Estado original"
        )

    def tearDown(self) -> None:
        self.container.fechar()

    def test_fachada_encaminha_operacoes_por_commands(self) -> None:
        atualizada = self.container.facade.atualizar_anotacao(
            self.anotacao.id, "Estado atualizado"
        )
        self.assertEqual(atualizada.conteudo, "Estado atualizado")
        self.assertEqual(
            self.container.facade.obter_anotacao(self.anotacao.id).conteudo,
            "Estado atualizado",
        )
        self.assertEqual(
            [item.id for item in self.container.facade.listar_anotacoes()],
            [self.anotacao.id],
        )

    def test_desfaz_uma_vez_a_ultima_atualizacao(self) -> None:
        data_original = self.anotacao.atualizada_em
        self.container.facade.atualizar_anotacao(
            self.anotacao.id, "Estado atualizado"
        )

        restaurada = self.container.facade.desfazer_ultima_atualizacao_anotacao()

        self.assertEqual(restaurada.conteudo, "Estado original")
        self.assertEqual(restaurada.atualizada_em, data_original)
        with self.assertRaises(SemAtualizacaoParaDesfazer):
            self.container.facade.desfazer_ultima_atualizacao_anotacao()

    def test_somente_a_atualizacao_mais_recente_fica_disponivel(self) -> None:
        outra = self.container.facade.criar_anotacao(
            "10.0.0.10", "Outra anotação"
        )
        self.container.facade.atualizar_anotacao(self.anotacao.id, "Primeira mudança")
        self.container.facade.atualizar_anotacao(outra.id, "Última mudança")

        restaurada = self.container.facade.desfazer_ultima_atualizacao_anotacao()

        self.assertEqual(restaurada.id, outra.id)
        self.assertEqual(restaurada.conteudo, "Outra anotação")
        self.assertEqual(
            self.container.facade.obter_anotacao(self.anotacao.id).conteudo,
            "Primeira mudança",
        )

    def test_memento_da_atualizacao_mais_recente_sobrevive_ao_reinicio(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            caminho = Path(temp_dir) / "nethub.db"
            primeiro = criar_container(caminho)
            primeiro.inventario.registrar_descoberta(Host("10.0.0.20"))
            anotacao = primeiro.facade.criar_anotacao("10.0.0.20", "Antes")
            primeiro.facade.atualizar_anotacao(anotacao.id, "Depois")
            primeiro.fechar()

            segundo = criar_container(caminho)
            restaurada = segundo.facade.desfazer_ultima_atualizacao_anotacao()
            self.assertEqual(restaurada.conteudo, "Antes")
            segundo.fechar()

    def test_atualizacao_invalida_nao_cria_memento(self) -> None:
        with self.assertRaises(ValueError):
            self.container.facade.atualizar_anotacao(self.anotacao.id, "  ")
        with self.assertRaises(SemAtualizacaoParaDesfazer):
            self.container.facade.desfazer_ultima_atualizacao_anotacao()

    def test_cli_disponibiliza_desfazer_pela_fachada(self) -> None:
        self.container.facade.atualizar_anotacao(self.anotacao.id, "Alterada")
        cli = CliAnotacoes(self.container.facade)
        saida = StringIO()

        with redirect_stdout(saida):
            codigo = cli.despachar(Namespace(anotacoes_cmd="desfazer"))

        self.assertEqual(codigo, 0)
        self.assertIn("Estado original", saida.getvalue())


if __name__ == "__main__":
    unittest.main()
