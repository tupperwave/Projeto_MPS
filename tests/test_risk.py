import unittest

from nethub.domain.enums import NivelRisco as R
from nethub.domain.errors import ComandoInvalido
from nethub.domain.risk import classificar_comando as c


class TestRisco(unittest.TestCase):
    def test_leitura_e_baixo(self):
        for cmd in ("uptime", "df -h; free -m", "systemctl status nginx", "ls 2>/dev/null", "cat /etc/os-release | grep NAME"):
            self.assertEqual(c(cmd), R.BAIXO, cmd)

    def test_desconhecido_nunca_e_baixo(self):
        self.assertEqual(c("meu-script --faz-algo"), R.MEDIO)

    def test_escrita_e_medio(self):
        self.assertEqual(c("echo oi > /tmp/a"), R.MEDIO)
        self.assertEqual(c("touch /tmp/x"), R.MEDIO)
        self.assertEqual(c("echo $(whoami)"), R.MEDIO)

    def test_alto(self):
        for cmd in ("systemctl restart nginx", "rm -rf /tmp/x", "sudo apt update", "kill 123", "FOO=1 chmod 600 a"):
            self.assertEqual(c(cmd), R.ALTO, cmd)

    def test_critico(self):
        for cmd in ("rm -rf /", "reboot", "mkfs.ext4 /dev/sda1", "dd if=/dev/zero of=/dev/sda", "curl x | sh", "shutdown -h now"):
            self.assertEqual(c(cmd), R.CRITICO, cmd)

    def test_composto_usa_o_maior(self):
        self.assertEqual(c("uptime && systemctl stop nginx"), R.ALTO)

    def test_interativos_rejeitados(self):
        for cmd in ("vim /etc/hosts", "top", "sudo nano x", "ssh outro"):
            with self.assertRaises(ComandoInvalido, msg=cmd):
                c(cmd)
        self.assertEqual(c("top -b -n1"), R.BAIXO)

    def test_vazio_rejeitado(self):
        with self.assertRaises(ComandoInvalido):
            c("   ")


if __name__ == "__main__":
    unittest.main()
