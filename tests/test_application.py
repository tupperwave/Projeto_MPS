import unittest

from nethub.bootstrap import criar_container
from nethub.domain.actions import ExecutarComandoAcao
from nethub.domain.enums import StatusCapacidade, StatusExecucao, TipoAcao, TipoCapacidade
from nethub.domain.errors import (
    AcaoIncompativel, DryRunObrigatorio, HostNaoEncontrado, NaoAutorizado, ScannerFalhou,
    ScannerIndisponivel, TagInvalida,
)
from nethub.domain.models import AlvoRede, FiltroHistorico
from tests.fakes import Confirma, FakeRemoto, FakeScanner, host_ssh

ALVO = AlvoRede.criar("10.0.0.0/24")


def novo(scanner=None, remoto=None):
    return criar_container(":memory:", scanner=scanner or FakeScanner(), remoto=remoto or FakeRemoto())


class TestDescoberta(unittest.TestCase):
    def test_exige_autorizacao(self):
        s = FakeScanner([host_ssh()])
        c = novo(s)
        with self.assertRaises(NaoAutorizado):
            c.descoberta.descobrir(ALVO, autorizado=False)
        self.assertEqual(s.chamadas, 0)

    def test_popula_inventario_e_detecta_ssh(self):
        c = novo(FakeScanner([host_ssh("10.0.0.5"), host_ssh("10.0.0.6", ssh=False)]))
        r = c.descoberta.descobrir(ALVO, True)
        self.assertEqual((r.novos, r.atualizados), (2, 0))
        h5, h6 = c.inventario.obter("10.0.0.5"), c.inventario.obter("10.0.0.6")
        self.assertEqual(h5.status_capacidade(TipoCapacidade.SSH), StatusCapacidade.DISPONIVEL)
        self.assertEqual(h6.status_capacidade(TipoCapacidade.SSH), StatusCapacidade.DESCONHECIDA)
        self.assertIsNotNone(h5.ultima_descoberta)

    def test_redescoberta_preserva_tags_e_primeira_data(self):
        c = novo(FakeScanner([host_ssh()]))
        c.descoberta.descobrir(ALVO, True)
        c.inventario.adicionar_tag("10.0.0.5", "Producao")
        antes = c.inventario.obter("10.0.0.5")
        r = c.descoberta.descobrir(ALVO, True)
        depois = c.inventario.obter("10.0.0.5")
        self.assertEqual((r.novos, r.atualizados), (0, 1))
        self.assertEqual(depois.tags, {"producao"})
        self.assertEqual(depois.id, antes.id)
        self.assertEqual(depois.primeira_descoberta, antes.primeira_descoberta)
        self.assertGreaterEqual(depois.ultima_descoberta, antes.ultima_descoberta)

    def test_falhas_viram_evidencia_e_nao_populam(self):
        for scanner, exc in ((FakeScanner(disponivel=False), ScannerIndisponivel), (FakeScanner(erro="xml ruim"), ScannerFalhou)):
            c = novo(scanner)
            with self.assertRaises(exc):
                c.descoberta.descobrir(ALVO, True)
            self.assertEqual(c.inventario.listar(), [])
            ev = c.historico.consultar()[0]
            self.assertEqual((ev.tipo_acao, ev.status), (TipoAcao.DESCOBERTA, StatusExecucao.FALHA))

    def test_sem_hosts(self):
        c = novo(FakeScanner([]))
        self.assertEqual(c.descoberta.descobrir(ALVO, True).hosts, [])
        self.assertIn("sem hosts", c.historico.consultar()[0].saida)


class TestTags(unittest.TestCase):
    def test_tags(self):
        c = novo(FakeScanner([host_ssh()]))
        c.descoberta.descobrir(ALVO, True)
        c.inventario.adicionar_tag("10.0.0.5", "web")
        self.assertEqual(len(c.inventario.listar("web")), 1)
        self.assertEqual(c.inventario.listar("db"), [])
        c.inventario.remover_tag("10.0.0.5", "web")
        self.assertEqual(c.inventario.listar("web"), [])
        with self.assertRaises(TagInvalida):
            c.inventario.adicionar_tag("10.0.0.5", "  ")
        with self.assertRaises(HostNaoEncontrado):
            c.inventario.adicionar_tag("1.1.1.1", "x")


class TestExecucao(unittest.TestCase):
    def setUp(self):
        self.remoto = FakeRemoto()
        self.c = novo(FakeScanner([host_ssh()]), self.remoto)
        self.c.descoberta.descobrir(ALVO, True)
        self.host = self.c.inventario.obter("10.0.0.5")

    def test_baixo_risco_executa_sem_confirmar_e_gera_evidencia(self):
        conf = Confirma()
        r = self.c.executor.executar(ExecutarComandoAcao("uptime"), self.host, conf)
        self.assertTrue(r.resultado.sucesso)
        self.assertEqual(conf.pedidos, [])
        self.assertEqual(self.remoto.chamadas, [("10.0.0.5", "uptime", 30.0)])
        ev = self.c.historico.detalhar(r.evidencia.id)
        self.assertEqual((ev.alvo, ev.detalhes, ev.saida, ev.exit_code), ("10.0.0.5", "uptime", "ok\n", 0))

    def test_cancelamento_nao_executa_mas_registra(self):
        r = self.c.executor.executar(ExecutarComandoAcao("touch /tmp/x"), self.host, Confirma(False))
        self.assertEqual(r.resultado.status, StatusExecucao.CANCELADA)
        self.assertEqual(self.remoto.chamadas, [])
        self.assertEqual(self.c.historico.detalhar(r.evidencia.id).status, StatusExecucao.CANCELADA)

    def test_alto_exige_confirmacao_explicita(self):
        conf = Confirma()
        self.c.executor.executar(ExecutarComandoAcao("systemctl restart nginx"), self.host, conf)
        self.assertEqual(conf.pedidos[0].tipo.value, "explicita")

    def test_critico_exige_dry_run(self):
        acao = ExecutarComandoAcao("reboot")
        with self.assertRaises(DryRunObrigatorio):
            self.c.executor.executar(acao, self.host, Confirma())
        self.assertEqual(self.remoto.chamadas, [])
        plano = self.c.executor.simular(acao, self.host)
        self.assertTrue(plano.exige_dry_run)
        self.assertEqual(self.remoto.chamadas, [])  # simular não executa
        r = self.c.executor.executar(acao, self.host, Confirma(), dry_run_realizado=True)
        self.assertTrue(r.resultado.sucesso)

    def test_acao_incompativel_sem_ssh(self):
        c = novo(FakeScanner([host_ssh("10.0.0.9", ssh=False)]))
        c.descoberta.descobrir(ALVO, True)
        h = c.inventario.obter("10.0.0.9")
        self.assertEqual(c.menu.acoes_compativeis(h), [])
        self.assertEqual(c.menu.capacidades_nao_verificadas(h), [TipoCapacidade.SSH])
        with self.assertRaises(AcaoIncompativel):
            c.executor.executar(ExecutarComandoAcao("uptime"), h, Confirma())

    def test_menu_mostra_acao_com_ssh(self):
        self.assertEqual(self.c.menu.acoes_compativeis(self.host), [ExecutarComandoAcao])

    def test_falha_remota_registrada(self):
        from nethub.domain.models import ResultadoExecucao
        self.remoto.resultado = ResultadoExecucao(StatusExecucao.FALHA, "", "Permission denied", 255)
        r = self.c.executor.executar(ExecutarComandoAcao("uptime"), self.host, Confirma())
        self.assertFalse(r.resultado.sucesso)
        self.assertEqual(self.c.historico.detalhar(r.evidencia.id).exit_code, 255)

    def test_excecao_inesperada_vira_falha(self):
        def explode(*a, **k):
            raise RuntimeError("kaboom")
        self.remoto.executar = explode
        r = self.c.executor.executar(ExecutarComandoAcao("uptime"), self.host, Confirma())
        self.assertEqual(r.resultado.status, StatusExecucao.FALHA)
        self.assertIn("kaboom", r.evidencia.erro)


class TestAuditoria(unittest.TestCase):
    def test_cadeia_integra_e_deteccao_de_adulteracao(self):
        c = novo(FakeScanner([host_ssh()]))
        c.descoberta.descobrir(ALVO, True)
        for cmd in ("uptime", "df -h", "whoami"):
            c.executor.executar(ExecutarComandoAcao(cmd), c.inventario.obter("10.0.0.5"), Confirma())
        self.assertEqual(c.evidencias.verificar_cadeia(), (True, None))
        c.banco.executar("UPDATE evidencias SET saida = 'adulterado' WHERE id = 2")
        self.assertEqual(c.evidencias.verificar_cadeia(), (False, 2))

    def test_filtros_de_historico(self):
        c = novo(FakeScanner([host_ssh()]))
        c.descoberta.descobrir(ALVO, True)
        c.executor.executar(ExecutarComandoAcao("uptime"), c.inventario.obter("10.0.0.5"), Confirma())
        self.assertEqual(len(c.historico.consultar(FiltroHistorico(tipo_acao=TipoAcao.COMANDO_REMOTO))), 1)
        self.assertEqual(len(c.historico.consultar(FiltroHistorico(alvo="10.0.0.0/24"))), 1)
        self.assertEqual(len(c.historico.consultar(FiltroHistorico(status=StatusExecucao.FALHA))), 0)
        self.assertEqual(len(c.historico.consultar()), 2)


if __name__ == "__main__":
    unittest.main()
