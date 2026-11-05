"""Executor remoto via binário `ssh` do sistema (agentless, RN-07).

MVP: autenticação por chave/agente (BatchMode=yes, nunca pede senha, então nenhuma credencial
passa pelo NetHub - RNF-03/RN-06). Um adaptador Paramiko pode implementar a mesma porta depois.
"""
from __future__ import annotations

import re
import subprocess
import time
from dataclasses import dataclass
from typing import Callable

from ..domain.enums import StatusExecucao
from ..domain.models import Host, ResultadoExecucao
from ..domain.ports import RemoteExecutorPort

_USUARIO_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_.-]{0,31}$")


@dataclass(frozen=True)
class ContextoAcesso:
    """Dados de acesso usados por uma execução remota."""

    usuario: str | None = None
    identidade: str | None = None

    def __post_init__(self) -> None:
        if self.usuario is not None and not _USUARIO_RE.match(self.usuario):
            raise ValueError(f"Nome de usuário SSH inválido: {self.usuario!r}")


class SshExecutor(RemoteExecutorPort):
    def __init__(
        self,
        usuario: str | None = None,
        identidade: str | None = None,
        contexto: ContextoAcesso | None = None,
        connect_timeout_s: int = 10,
        strict_host_key: str = "accept-new",
        binario: str = "ssh",
        runner: Callable = subprocess.run,
    ) -> None:
        if contexto is not None and (usuario is not None or identidade is not None):
            raise ValueError("Informe contexto ou usuario/identidade, não ambos.")
        self._contexto = contexto or ContextoAcesso(usuario, identidade)
        self._connect_timeout_s = connect_timeout_s
        self._strict = strict_host_key
        self._binario = binario
        self._runner = runner

    def montar_comando(
        self, host: Host, comando: str, contexto: ContextoAcesso | None = None
    ) -> list[str]:
        contexto = contexto or self._contexto
        cmd = [
            self._binario,
            "-p", str(host.porta_ssh),
            "-o", "BatchMode=yes",
            "-o", f"ConnectTimeout={self._connect_timeout_s}",
            "-o", f"StrictHostKeyChecking={self._strict}",
        ]
        if contexto.identidade:
            cmd += ["-i", contexto.identidade]
        destino = f"{contexto.usuario}@{host.ip}" if contexto.usuario else host.ip
        return cmd + ["--", destino, comando]

    def executar(
        self,
        host: Host,
        comando: str,
        timeout_s: float,
        contexto: ContextoAcesso | None = None,
    ) -> ResultadoExecucao:
        if timeout_s <= 0:
            raise ValueError("O timeout deve ser maior que zero.")
        inicio = time.monotonic()
        try:
            proc = self._runner(
                self.montar_comando(host, comando, contexto),
                capture_output=True,
                text=True,
                timeout=timeout_s,
            )
        except subprocess.TimeoutExpired:
            return ResultadoExecucao(
                StatusExecucao.EXPIRADA,
                stderr=f"Tempo limite de {timeout_s:.0f}s excedido.",
                duracao_s=time.monotonic() - inicio,
            )
        except OSError as exc:
            return ResultadoExecucao(
                StatusExecucao.FALHA,
                stderr=f"Não foi possível executar o cliente SSH: {exc}",
                duracao_s=time.monotonic() - inicio,
            )
        duracao = time.monotonic() - inicio
        status = StatusExecucao.CONCLUIDA if proc.returncode == 0 else StatusExecucao.FALHA
        # exit 255 do ssh = falha de conexão/autenticação (ou o próprio comando saiu com 255).
        return ResultadoExecucao(status, proc.stdout or "", proc.stderr or "", proc.returncode, duracao)
