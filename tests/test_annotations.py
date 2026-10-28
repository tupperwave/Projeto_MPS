import unittest

from nethub.bootstrap import criar_container
from nethub.application.facade_controller import FacadeSingletonController
from nethub.domain.errors import RegistroNaoEncontrado
from nethub.domain.models import Host


class TestAnotacoesHost(unittest.TestCase):
    def setUp(self):
        self.container = criar_container(":memory:")
        self.container.inventario.registrar_descoberta(Host("10.0.0.10"))

    def test_ciclo_completo_de_crud(self):
        anotacao = self.container.anotacoes.criar("10.0.0.10", "Host da aplicação")
        self.assertIsNotNone(anotacao.id)
        self.assertEqual(self.container.anotacoes.obter(anotacao.id).conteudo, "Host da aplicação")
        self.assertEqual([a.id for a in self.container.anotacoes.listar("10.0.0.10")], [anotacao.id])

        atualizada = self.container.anotacoes.atualizar(anotacao.id, "Servidor principal")
        self.assertEqual(atualizada.conteudo, "Servidor principal")
        self.assertEqual(self.container.anotacoes.obter(anotacao.id).conteudo, "Servidor principal")
        self.assertTrue(self.container.anotacoes.excluir(anotacao.id))
        self.assertFalse(self.container.anotacoes.excluir(anotacao.id))
        with self.assertRaises(RegistroNaoEncontrado):
            self.container.anotacoes.obter(anotacao.id)

    def test_rejeita_host_inexistente_e_conteudo_vazio(self):
        with self.assertRaises(RegistroNaoEncontrado):
            self.container.anotacoes.criar("10.0.0.99", "Não cadastrado")
        with self.assertRaises(ValueError):
            self.container.anotacoes.criar("10.0.0.10", "   ")
        anotacao = self.container.anotacoes.criar("10.0.0.10", "Conteúdo preservado")
        with self.assertRaises(ValueError):
            self.container.anotacoes.atualizar(anotacao.id, "   ")
        self.assertEqual(self.container.anotacoes.obter(anotacao.id).conteudo, "Conteúdo preservado")

    def test_remover_host_apaga_anotacoes_vinculadas(self):
        self.container.anotacoes.criar("10.0.0.10", "Removida em cascata")
        self.container.banco.executar("DELETE FROM hosts WHERE ip = ?", ("10.0.0.10",))
        self.assertEqual(self.container.anotacoes.listar(), [])

    def test_contagem_da_fachada_reflete_hosts_usuarios_e_anotacoes(self):
        self.assertEqual(self.container.facade.quantidade_entidades_cadastradas(), 1)
        self.container.usuarios.cadastrar("Ana", "Abcd123!")
        anotacao = self.container.anotacoes.criar("10.0.0.10", "Contagem")
        self.assertEqual(self.container.facade.quantidade_entidades_cadastradas(), 3)
        self.container.anotacoes.excluir(anotacao.id)
        self.assertEqual(self.container.facade.quantidade_entidades_cadastradas(), 2)

    def test_contagem_vazia_retorna_zero(self):
        vazio = criar_container(":memory:")
        self.assertEqual(vazio.facade.quantidade_entidades_cadastradas(), 0)

    def test_facade_e_unica_para_a_composicao(self):
        segunda_referencia = FacadeSingletonController(
            self.container.inventario,
            self.container.descoberta,
            self.container.menu,
            self.container.executor,
            self.container.evidencias,
            self.container.historico,
            self.container.usuarios,
            self.container.anotacoes,
        )
        self.assertIs(segunda_referencia, self.container.facade)

    def test_composicoes_distintas_recebem_fachadas_distintas(self):
        outra_composicao = criar_container(":memory:")
        self.assertIsNot(outra_composicao.facade, self.container.facade)

    def test_fachada_expoe_os_gerentes_da_composicao(self):
        for nome in ("inventario", "descoberta", "menu", "executor", "evidencias", "historico", "usuarios", "anotacoes"):
            self.assertIs(getattr(self.container.facade, nome), getattr(self.container, nome))
