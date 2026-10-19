import io
import unittest
from contextlib import redirect_stderr, redirect_stdout

from nethub.bootstrap import criar_container
from nethub.presentation.cli import main
from tests.fakes import FakeRemoto, FakeScanner, host_ssh


def rodar(c, *argv, entrada=lambda _: ""):
    out, err = io.StringIO(), io.StringIO()
    with redirect_stdout(out), redirect_stderr(err):
        codigo = main(list(argv), container=c, entrada=entrada)
    return codigo, out.getvalue(), err.getvalue()


class TestCli(unittest.TestCase):
    def setUp(self):
        self.remoto = FakeRemoto()
        self.c = criar_container(":memory:", scanner=FakeScanner([host_ssh()]), remoto=self.remoto)

    def test_fluxo_completo(self):
        cod, out, _ = rodar(self.c, "descobrir", "10.0.0.0/24", "--autorizado")
        self.assertEqual(cod, 0); self.assertIn("10.0.0.5", out)
        cod, out, _ = rodar(self.c, "hosts"); self.assertIn("10.0.0.5", out)
        cod, out, _ = rodar(self.c, "host", "10.0.0.5")
        self.assertIn("Executar comando remoto", out); self.assertIn("desconhecida", out)
        cod, out, _ = rodar(self.c, "exec", "10.0.0.5", "--", "uptime")
        self.assertEqual(cod, 0); self.assertIn("ok", out); self.assertIn("Evidência registrada", out)
        cod, out, _ = rodar(self.c, "historico"); self.assertIn("comando_remoto", out)
        cod, out, _ = rodar(self.c, "historico", "--detalhe", "2"); self.assertIn("uptime", out)
        cod, out, _ = rodar(self.c, "verificar-auditoria"); self.assertEqual(cod, 0)

    def test_descobrir_sem_autorizacao_pergunta_e_bloqueia(self):
        cod, _, err = rodar(self.c, "descobrir", "10.0.0.0/24", entrada=lambda _: "n")
        self.assertEqual(cod, 2); self.assertIn("autorização", err)

    def test_exec_alto_risco_exige_ip(self):
        rodar(self.c, "descobrir", "10.0.0.0/24", "--autorizado")
        cod, out, _ = rodar(self.c, "exec", "10.0.0.5", "--", "systemctl", "restart", "nginx", entrada=lambda _: "errado")
        self.assertEqual(cod, 3); self.assertEqual(self.remoto.chamadas, [])
        cod, out, _ = rodar(self.c, "exec", "10.0.0.5", "--confirmar-ip", "10.0.0.5", "--", "systemctl", "restart", "nginx")
        self.assertEqual(cod, 0); self.assertEqual(len(self.remoto.chamadas), 1)

    def test_simular_nao_executa(self):
        rodar(self.c, "descobrir", "10.0.0.0/24", "--autorizado")
        cod, out, _ = rodar(self.c, "exec", "10.0.0.5", "--simular", "--", "reboot")
        self.assertEqual(cod, 0); self.assertIn("DRY-RUN", out); self.assertEqual(self.remoto.chamadas, [])

    def test_erros_amigaveis(self):
        self.assertEqual(rodar(self.c, "host", "1.2.3.4")[0], 2)
        self.assertEqual(rodar(self.c, "descobrir", "abc", "--autorizado")[0], 2)
        rodar(self.c, "descobrir", "10.0.0.0/24", "--autorizado")
        self.assertEqual(rodar(self.c, "exec", "10.0.0.5", "--", "vim", "x")[0], 2)

    def test_tags_e_historico_vazio(self):
        rodar(self.c, "descobrir", "10.0.0.0/24", "--autorizado")
        _, out, _ = rodar(self.c, "tag", "add", "10.0.0.5", "web"); self.assertIn("web", out)
        _, out, _ = rodar(self.c, "hosts", "--tag", "db"); self.assertIn("vazio", out)
        _, out, _ = rodar(self.c, "historico", "--host", "9.9.9.9"); self.assertIn("Nenhum registro", out)


if __name__ == "__main__":
    unittest.main()
