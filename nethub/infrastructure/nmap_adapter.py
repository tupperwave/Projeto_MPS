"""Adaptador do Nmap: executa a varredura, lê o XML gerado e converte em Hosts (RF-01)."""
from __future__ import annotations

import shutil
import subprocess
import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Callable

from ..domain.errors import ScannerFalhou, ScannerIndisponivel
from ..domain.models import AlvoRede, Host, PortaServico
from ..domain.ports import NetworkScannerPort

ARGS_PADRAO = ["-sV", "--top-ports", "100", "-T4", "--open"]


def interpretar_xml(texto: str) -> list[Host]:
    """Converte o XML do Nmap em hosts. Levanta ScannerFalhou se a varredura não terminou com sucesso."""
    try:
        raiz = ET.fromstring(texto)
    except ET.ParseError as exc:
        raise ScannerFalhou(f"XML do Nmap inválido: {exc}") from exc

    fim = raiz.find("runstats/finished")
    if fim is None or fim.get("exit") != "success":
        raise ScannerFalhou("Varredura incompleta: o Nmap não finalizou com sucesso.")

    hosts: list[Host] = []
    for no in raiz.findall("host"):
        status = no.find("status")
        if status is None or status.get("state") != "up":
            continue
        enderecos = {a.get("addrtype"): a.get("addr") for a in no.findall("address")}
        ip = enderecos.get("ipv4") or enderecos.get("ipv6")
        if not ip:
            continue
        nomes = no.findall("hostnames/hostname")
        preferido = next((n for n in nomes if n.get("type") == "user"), nomes[0] if nomes else None)
        host = Host(ip=ip, hostname=(preferido.get("name") or "") if preferido is not None else "")
        for porta in no.findall("ports/port"):
            estado = porta.find("state")
            servico = porta.find("service")
            host.portas.append(
                PortaServico(
                    numero=int(porta.get("portid", "0")),
                    protocolo=porta.get("protocol", "tcp"),
                    estado=estado.get("state", "") if estado is not None else "",
                    servico=(servico.get("name") or "") if servico is not None else "",
                    versao=" ".join(
                        filter(None, [servico.get("product"), servico.get("version")])
                    )
                    if servico is not None
                    else "",
                )
            )
        hosts.append(host)
    return hosts


class NmapAdapter(NetworkScannerPort):
    def __init__(
        self,
        binario: str = "nmap",
        args: list[str] | None = None,
        timeout_s: float = 900,
        runner: Callable = subprocess.run,
        localizador: Callable[[str], str | None] = shutil.which,
    ) -> None:
        self._binario = binario
        self._args = list(ARGS_PADRAO if args is None else args)
        self._timeout_s = timeout_s
        self._runner = runner
        self._localizador = localizador

    def disponivel(self) -> bool:
        return self._localizador(self._binario) is not None

    def varrer(self, alvo: AlvoRede) -> list[Host]:
        if not self.disponivel():
            raise ScannerIndisponivel("Nmap não encontrado no PATH.")
        with tempfile.TemporaryDirectory(prefix="nethub-") as tmp:
            arquivo = Path(tmp) / "varredura.xml"
            comando = [self._binario, *self._args, "-oX", str(arquivo), alvo.cidr]
            try:
                proc = self._runner(comando, capture_output=True, text=True, timeout=self._timeout_s)
            except subprocess.TimeoutExpired as exc:
                raise ScannerFalhou(f"Tempo limite da varredura excedido ({self._timeout_s:.0f}s).") from exc
            except OSError as exc:
                raise ScannerFalhou(f"Não foi possível executar o Nmap: {exc}") from exc
            if proc.returncode != 0:
                detalhe = (proc.stderr or "").strip().splitlines()
                raise ScannerFalhou(f"Nmap retornou código {proc.returncode}: {detalhe[-1] if detalhe else 'sem detalhes'}")
            if not arquivo.exists():
                raise ScannerFalhou("O Nmap não gerou o arquivo XML esperado.")
            return interpretar_xml(arquivo.read_text(encoding="utf-8"))
