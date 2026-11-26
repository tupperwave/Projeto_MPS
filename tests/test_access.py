"""Testes de registro e consulta das tentativas de acesso."""
from __future__ import annotations

import unittest

from nethub.bootstrap import criar_container
from nethub.application.access_report import PlainTextAccessReport


class TestAcessoUsuario(unittest.TestCase):
    def setUp(self) -> None:
        self.container = criar_container(":memory:")
        self.container.usuarios.cadastrar("Ana", "Senha123!")

    def tearDown(self) -> None:
        self.container.fechar()

    def test_autenticacao_valida_registra_acesso_bem_sucedido(self) -> None:
        self.assertTrue(self.container.usuarios.autenticar("Ana", "Senha123!"))

        relatorio = self.container.relatorios_acesso.gerar(
            PlainTextAccessReport()
        ).decode("utf-8")
        self.assertIn("login='Ana'", relatorio)
        self.assertIn("total=1, sucessos=1, falhas=0", relatorio)

    def test_senha_incorreta_registra_falha_sem_autenticar(self) -> None:
        self.assertFalse(self.container.usuarios.autenticar("Ana", "Errada123!"))

        relatorio = self.container.relatorios_acesso.gerar(
            PlainTextAccessReport()
        ).decode("utf-8")
        self.assertIn("total=1, sucessos=0, falhas=1", relatorio)

    def test_autenticacao_sem_usuario_registra_falha(self) -> None:
        self.assertFalse(self.container.usuarios.autenticar("Bia", "Senha123!"))

        relatorio = self.container.relatorios_acesso.gerar(
            PlainTextAccessReport()
        ).decode("utf-8")
        self.assertIn("login='Bia'", relatorio)
        self.assertIn("falhas=1", relatorio)


if __name__ == "__main__":
    unittest.main()
