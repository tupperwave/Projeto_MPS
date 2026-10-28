"""Detecção da rede IPv4 ligada à rota padrão no Linux."""
from __future__ import annotations

import ipaddress
import subprocess

from ..domain.errors import RedeLocalIndisponivel
from ..domain.models import AlvoRede
from ..domain.ports import LocalNetworkDetectorPort


class LinuxLocalNetworkDetector(LocalNetworkDetectorPort):
    """Usa iproute para identificar a interface e o CIDR da rota IPv4 padrão."""

    def detectar(self) -> AlvoRede:
        try:
            rotas = subprocess.run(
                ["ip", "-4", "route", "show", "default"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
        except FileNotFoundError as exc:
            raise RedeLocalIndisponivel(
                "O utilitário 'ip' não foi encontrado. Instale iproute ou informe um CIDR manualmente."
            ) from exc
        except (OSError, subprocess.TimeoutExpired) as exc:
            raise RedeLocalIndisponivel(
                "Não foi possível consultar a rota padrão IPv4. Informe um CIDR manualmente."
            ) from exc

        interface = self._interface_da_rota(rotas.stdout)
        try:
            enderecos = subprocess.run(
                ["ip", "-o", "-4", "address", "show", "dev", interface, "scope", "global"],
                capture_output=True,
                text=True,
                timeout=5,
                check=False,
            )
        except (OSError, subprocess.TimeoutExpired) as exc:
            raise RedeLocalIndisponivel(
                "Não foi possível consultar o endereço IPv4 da interface padrão. Informe um CIDR manualmente."
            ) from exc

        for linha in enderecos.stdout.splitlines():
            partes = linha.split()
            try:
                endereco = partes[partes.index("inet") + 1]
            except (ValueError, IndexError):
                continue
            try:
                rede = ipaddress.ip_interface(endereco).network
            except ValueError:
                continue
            return AlvoRede.criar(str(rede))

        raise RedeLocalIndisponivel(
            f"A interface padrão {interface!r} não tem endereço IPv4 global. Informe um CIDR manualmente."
        )

    @staticmethod
    def _interface_da_rota(saida: str) -> str:
        for linha in saida.splitlines():
            partes = linha.split()
            try:
                return partes[partes.index("dev") + 1]
            except (ValueError, IndexError):
                continue
        raise RedeLocalIndisponivel(
            "Não foi encontrada uma rota padrão IPv4. Informe um CIDR manualmente."
        )
