"""Executor remoto via binário `ssh` do sistema (agentless, RN-07).

MVP: autenticação por chave/agente (BatchMode=yes, nunca pede senha, então nenhuma credencial
passa pelo NetHub - RNF-03/RN-06). Um adaptador Paramiko pode implementar a mesma porta depois.
"""
from __future__ import annotations

import re
import subprocess
import time
from typing import Callable

from ..domain.enums import StatusExecucao
from ..domain.models import Host, ResultadoExecucao
from ..domain.ports import RemoteExecutorPort

_USUARIO_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_.-]{0,31}$")


class SshExecutor(RemoteExecutorPort):
    def __init__(
        self,
        usuario: str | None = None,
        identidade: str | None = None,
        connect_timeout_s: int = 10,
        strict_host_key: str = "accept-new",
        binario: str = "ssh",
        runner: Callable = subprocess.run,
    ) -> None:
        if usuario is not None and not _USUARIO_RE.match(usuario):
            raise ValueError(f"Nome de usuário SSH inválido: {usuario!r}")
        self._usuario = usuario
        self._identidade = identidade
        self._connect_timeout_s = connect_timeout_s
        self._strict = strict_host_key
        self._binario = binario
        self._runner = runner

    def montar_comando(self, host: Host, comando: str) -> list[str]:
        cmd = [
            self._binario,
            "-p", str(host.porta_ssh),
            "-o", "BatchMode=yes",
            "-o", f"ConnectTimeout={self._connect_timeout_s}",
            "-o", f"StrictHostKeyChecking={self._strict}",
        ]
        if self._identidade:
            cmd += ["-i", self._identidade]
        destino = f"{self._usuario}@{host.ip}" if self._usuario else host.ip
        return cmd + ["--", destino, comando]

    def executar(self, host: Host, comando: str, timeout_s: float) -> ResultadoExecucao:
        inicio = time.monotonic()
        try:
            proc = self._runner(
                self.montar_comando(host, comando), capture_output=True, text=True, timeout=timeout_s
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
