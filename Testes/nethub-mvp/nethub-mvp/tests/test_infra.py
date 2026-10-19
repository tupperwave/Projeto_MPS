import subprocess
import unittest

from nethub.domain.enums import StatusExecucao
from nethub.domain.errors import ScannerFalhou, ScannerIndisponivel
from nethub.domain.models import AlvoRede, Host
from nethub.infrastructure.nmap_adapter import NmapAdapter, interpretar_xml
from nethub.infrastructure.ssh_adapter import SshExecutor
from tests.fakes import CompletedFake, XML_EXEMPLO, runner_nmap_ok, runner_timeout


class TestNmap(unittest.TestCase):
    def test_interpretar_xml(self):
        hosts = interpretar_xml(XML_EXEMPLO)
        self.assertEqual([h.ip for h in hosts], ["192.168.0.1", "192.168.0.5"])  # host 'down' ignorado
        self.assertEqual(hosts[0].hostname, "router.local")
        self.assertEqual([p.numero for p in hosts[0].portas], [22, 80])
        self.assertEqual(hosts[0].portas[0].versao, "OpenSSH 9.2")

    def test_xml_invalido_ou_incompleto(self):
        with self.assertRaises(ScannerFalhou):
            interpretar_xml("<nmaprun><host>")
        with self.assertRaises(ScannerFalhou):
            interpretar_xml("<nmaprun><host><status state='up'/></host></nmaprun>")  # sem runstats

    def test_varrer_ok(self):
        a = NmapAdapter(runner=runner_nmap_ok, localizador=lambda _: "/usr/bin/nmap")
        self.assertEqual(len(a.varrer(AlvoRede.criar("192.168.0.0/29"))), 2)

    def test_indisponivel(self):
        a = NmapAdapter(localizador=lambda _: None)
        self.assertFalse(a.disponivel())
        with self.assertRaises(ScannerIndisponivel):
            a.varrer(AlvoRede.criar("10.0.0.1"))

    def test_falhas(self):
        alvo = AlvoRede.criar("10.0.0.0/30")
        loc = lambda _: "/x/nmap"
        with self.assertRaises(ScannerFalhou):
            NmapAdapter(runner=lambda c, **k: CompletedFake(1, "", "boom"), localizador=loc).varrer(alvo)
        with self.assertRaises(ScannerFalhou):
            NmapAdapter(runner=runner_timeout, localizador=loc).varrer(alvo)
        with self.assertRaises(ScannerFalhou):  # retornou 0 mas não gerou XML
            NmapAdapter(runner=lambda c, **k: CompletedFake(0), localizador=loc).varrer(alvo)

    def test_alvo_invalido(self):
        from nethub.domain.errors import AlvoInvalido
        for ruim in ("", "abc", "10.0.0.0/8", "999.1.1.1"):
            with self.assertRaises(AlvoInvalido, msg=ruim):
                AlvoRede.criar(ruim)


class TestSsh(unittest.TestCase):
    host = Host(ip="10.0.0.5", porta_ssh=2222)

    def test_comando_montado(self):
        cmd = SshExecutor(usuario="admin", identidade="/k").montar_comando(self.host, "uptime")
        self.assertIn("BatchMode=yes", cmd)
        self.assertEqual(cmd[-3:], ["--", "admin@10.0.0.5", "uptime"])
        self.assertEqual(cmd[cmd.index("-p") + 1], "2222")

    def test_usuario_invalido(self):
        with self.assertRaises(ValueError):
            SshExecutor(usuario="-oProxyCommand=x")

    def test_sucesso_falha_timeout(self):
        ok = SshExecutor(runner=lambda c, **k: CompletedFake(0, "oi\n", "")).executar(self.host, "x", 5)
        self.assertEqual((ok.status, ok.stdout, ok.exit_code), (StatusExecucao.CONCLUIDA, "oi\n", 0))
        falha = SshExecutor(runner=lambda c, **k: CompletedFake(255, "", "Permission denied")).executar(self.host, "x", 5)
        self.assertEqual((falha.status, falha.exit_code), (StatusExecucao.FALHA, 255))
        exp = SshExecutor(runner=runner_timeout).executar(self.host, "x", 5)
        self.assertEqual(exp.status, StatusExecucao.EXPIRADA)

        def sem_ssh(c, **k):
            raise FileNotFoundError("ssh")
        self.assertEqual(SshExecutor(runner=sem_ssh).executar(self.host, "x", 5).status, StatusExecucao.FALHA)


if __name__ == "__main__":
    unittest.main()
