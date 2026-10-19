"""Dublês de teste para as portas do NetHub."""
import subprocess

from nethub.domain.enums import StatusExecucao
from nethub.domain.errors import ScannerFalhou
from nethub.domain.models import Host, PortaServico, ResultadoExecucao
from nethub.domain.ports import ConfirmationPort, NetworkScannerPort, RemoteExecutorPort

XML_EXEMPLO = """<?xml version="1.0"?>
<nmaprun scanner="nmap" args="nmap -sV 192.168.0.0/29">
<host><status state="up"/><address addr="192.168.0.1" addrtype="ipv4"/>
<hostnames><hostname name="router.local" type="PTR"/></hostnames>
<ports>
<port protocol="tcp" portid="22"><state state="open"/><service name="ssh" product="OpenSSH" version="9.2"/></port>
<port protocol="tcp" portid="80"><state state="open"/><service name="http" product="nginx" version="1.22"/></port>
</ports></host>
<host><status state="up"/><address addr="192.168.0.5" addrtype="ipv4"/><ports>
<port protocol="tcp" portid="443"><state state="open"/><service name="https"/></port></ports></host>
<host><status state="down"/><address addr="192.168.0.9" addrtype="ipv4"/></host>
<runstats><finished exit="success"/><hosts up="2" down="1" total="3"/></runstats>
</nmaprun>"""


def host_ssh(ip="10.0.0.5", ssh=True) -> Host:
    h = Host(ip=ip, portas=[PortaServico(22, servico="ssh")] if ssh else [PortaServico(80, servico="http")])
    return h


class FakeScanner(NetworkScannerPort):
    def __init__(self, hosts=None, disponivel=True, erro=None):
        self.hosts, self._disp, self.erro, self.chamadas = hosts or [], disponivel, erro, 0

    def disponivel(self):
        return self._disp

    def varrer(self, alvo):
        self.chamadas += 1
        if self.erro:
            raise ScannerFalhou(self.erro)
        return list(self.hosts)


class FakeRemoto(RemoteExecutorPort):
    def __init__(self, resultado=None):
        self.resultado = resultado or ResultadoExecucao(StatusExecucao.CONCLUIDA, "ok\n", "", 0, 0.01)
        self.chamadas = []

    def executar(self, host, comando, timeout_s):
        self.chamadas.append((host.ip, comando, timeout_s))
        return self.resultado


class Confirma(ConfirmationPort):
    def __init__(self, resposta=True):
        self.resposta, self.pedidos = resposta, []

    def confirmar(self, pedido):
        self.pedidos.append(pedido)
        return self.resposta


class CompletedFake:
    def __init__(self, returncode=0, stdout="", stderr=""):
        self.returncode, self.stdout, self.stderr = returncode, stdout, stderr


def runner_nmap_ok(cmd, **kw):
    """Simula o nmap escrevendo o XML no caminho passado em -oX."""
    caminho = cmd[cmd.index("-oX") + 1]
    with open(caminho, "w", encoding="utf-8") as f:
        f.write(XML_EXEMPLO)
    return CompletedFake(0)


def runner_timeout(cmd, **kw):
    raise subprocess.TimeoutExpired(cmd, kw.get("timeout", 1))
