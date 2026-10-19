"""Classificação de risco de comandos remotos (RF-04, RNF-04).

IMPORTANTE: é uma heurística *conservadora* para decidir o nível de confirmação,
não uma barreira de segurança. Comandos desconhecidos são tratados como risco MÉDIO
(nunca como BAIXO) e padrões destrutivos conhecidos são CRÍTICOS.
"""
from __future__ import annotations

import posixpath
import re
import shlex

from .enums import NivelRisco
from .errors import ComandoInvalido

_SEPARADORES = re.compile(r"\|\||&&|;|\||\n")
_ENV_ASSIGN = re.compile(r"^[A-Za-z_]\w*=")
_SUBSTITUICAO = re.compile(r"\$\(|`")
_REDIRECIONAMENTOS_INOFENSIVOS = re.compile(r"(\d?|&)>+\s*/dev/null|\d>&\d")

_CRITICOS = [
    re.compile(p)
    for p in (
        r"\brm\s+.*-\w*[rR]\w*.*\s(/|/\*|~/?|\*)(\s|$)",
        r"\bmkfs(\.\w+)?\b",
        r"\bwipefs\b",
        r"\bdd\b.*\bof=/dev/",
        r":\(\)\s*\{",
        r"\b(shutdown|reboot|halt|poweroff)\b",
        r"\binit\s+[06]\b",
        r">\s*/dev/(sd|nvme|vd|xvd)",
        r"\b(chmod|chown)\s+(-\S+\s+)*-\w*R\w*\b.*\s/(\s|$)",
        r"\|\s*(sudo\s+)?(ba|z|da)?sh\b",
    )
]

_LEITURA = frozenset(
    {
        "uptime", "df", "free", "ls", "cat", "head", "tail", "whoami", "id", "hostname",
        "uname", "date", "pwd", "ps", "ss", "ip", "echo", "wc", "du", "lsblk", "grep",
        "stat", "printenv", "lscpu", "w", "who", "last", "journalctl", "dmesg", "which",
        "sort", "uniq", "top", "true", "false", "nproc", "vmstat", "netstat", "ping", "dig", "getent",
    }
)
_ALTO = frozenset(
    {
        "rm", "mv", "kill", "pkill", "killall", "chmod", "chown", "chgrp", "apt", "apt-get",
        "dpkg", "yum", "dnf", "apk", "zypper", "pacman", "useradd", "userdel", "usermod",
        "groupadd", "iptables", "ip6tables", "nft", "ufw", "firewall-cmd", "mount", "umount",
        "crontab", "service", "swapoff", "modprobe", "rmmod", "sysctl", "visudo",
    }
)
_MEDIO = frozenset({"cp", "mkdir", "touch", "tee", "ln", "wget", "curl", "sed"})

_SYSTEMCTL_LEITURA = frozenset(
    {"status", "is-active", "is-enabled", "is-failed", "list-units", "list-unit-files", "list-timers", "show", "cat"}
)
_SYSTEMCTL_ALTO = frozenset(
    {"stop", "restart", "reload", "disable", "enable", "mask", "kill", "daemon-reload", "isolate", "set-default"}
)

_INTERATIVOS = frozenset(
    {"vi", "vim", "nano", "emacs", "less", "more", "man", "htop", "watch", "ssh", "telnet",
     "ftp", "sftp", "passwd", "su", "screen", "tmux"}
)


def _segmentos(comando: str) -> list[str]:
    return [s.strip() for s in _SEPARADORES.split(comando) if s.strip()]


def _tokens(segmento: str) -> list[str]:
    try:
        return shlex.split(segmento)
    except ValueError:
        return segmento.split()


def _normalizar(tokens: list[str]) -> tuple[bool, list[str]]:
    """Remove atribuições de ambiente e prefixo sudo/doas. Retorna (usa_sudo, tokens)."""
    i = 0
    while i < len(tokens) and _ENV_ASSIGN.match(tokens[i]):
        i += 1
    tokens = tokens[i:]
    usa_sudo = False
    if tokens and tokens[0] in ("sudo", "doas"):
        usa_sudo = True
        tokens = [t for t in tokens[1:] if not t.startswith("-")]
    return usa_sudo, tokens


def validar_comando_nao_interativo(comando: str) -> None:
    if comando is None or not comando.strip():
        raise ComandoInvalido("Comando vazio.")
    if "\x00" in comando:
        raise ComandoInvalido("Comando contém caractere nulo.")
    for seg in _segmentos(comando):
        _, tokens = _normalizar(_tokens(seg))
        if not tokens:
            continue
        nome = posixpath.basename(tokens[0])
        if nome in _INTERATIVOS or (nome == "top" and "-b" not in tokens):
            raise ComandoInvalido(
                f"'{nome}' é interativo; o NetHub só executa comandos não interativos (use 'top -b', por exemplo)."
            )


def _nivel_segmento(tokens: list[str]) -> NivelRisco:
    usa_sudo, tokens = _normalizar(tokens)
    base = NivelRisco.ALTO if usa_sudo else NivelRisco.BAIXO
    if not tokens:
        return base
    nome = posixpath.basename(tokens[0])
    if nome == "systemctl":
        sub = next((t for t in tokens[1:] if not t.startswith("-")), "")
        if sub in _SYSTEMCTL_LEITURA:
            nivel = NivelRisco.BAIXO
        elif sub in _SYSTEMCTL_ALTO:
            nivel = NivelRisco.ALTO
        else:
            nivel = NivelRisco.MEDIO
    elif nome in _ALTO:
        nivel = NivelRisco.ALTO
    elif nome in _LEITURA:
        nivel = NivelRisco.BAIXO
    else:  # _MEDIO e desconhecidos
        nivel = NivelRisco.MEDIO
    return max(base, nivel)


def classificar_comando(comando: str) -> NivelRisco:
    validar_comando_nao_interativo(comando)
    if any(p.search(comando) for p in _CRITICOS):
        return NivelRisco.CRITICO
    nivel = NivelRisco.BAIXO
    for seg in _segmentos(comando):
        nivel = max(nivel, _nivel_segmento(_tokens(seg)))
    if _SUBSTITUICAO.search(comando):
        nivel = max(nivel, NivelRisco.MEDIO)
    sem_inofensivos = _REDIRECIONAMENTOS_INOFENSIVOS.sub("", comando)
    if ">" in sem_inofensivos:
        nivel = max(nivel, NivelRisco.MEDIO)
    return nivel
