"""Entidades e objetos de valor do domínio."""
from __future__ import annotations

import hashlib
import ipaddress
import json
import re
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone

from .enums import NivelRisco, StatusCapacidade, StatusExecucao, TipoAcao, TipoCapacidade
from .errors import AlvoInvalido, TagInvalida

MAX_SAIDA = 64 * 1024
_MARCA_TRUNCADO = "\n[... saída truncada pelo NetHub ...]"
_TAG_RE = re.compile(r"^[a-z0-9][a-z0-9_.-]{0,31}$")


# --------------------------------------------------------------------- utilidades
def agora() -> datetime:
    return datetime.now(timezone.utc)


def para_iso(dt: datetime) -> str:
    """Formato único (UTC, microssegundos) para persistência e hash."""
    return dt.astimezone(timezone.utc).isoformat(timespec="microseconds")


def de_iso(texto: str) -> datetime:
    return datetime.fromisoformat(texto)


def truncar(texto: str) -> str:
    if len(texto) <= MAX_SAIDA:
        return texto
    return texto[:MAX_SAIDA] + _MARCA_TRUNCADO


def normalizar_tag(tag: str) -> str:
    valor = (tag or "").strip().lower()
    if not _TAG_RE.match(valor):
        raise TagInvalida(
            "Tag inválida: use 1-32 caracteres (letras, números, '_', '.', '-'), começando por letra/número."
        )
    return valor


# ------------------------------------------------------------------------- rede
@dataclass(frozen=True)
class AlvoRede:
    cidr: str

    @classmethod
    def criar(cls, texto: str, max_enderecos: int = 65536) -> "AlvoRede":
        try:
            rede = ipaddress.ip_network((texto or "").strip(), strict=False)
        except ValueError as exc:
            raise AlvoInvalido(f"Intervalo de rede inválido: {texto!r}") from exc
        if rede.num_addresses > max_enderecos:
            raise AlvoInvalido(
                f"Intervalo muito amplo ({rede.num_addresses} endereços; máximo {max_enderecos})."
            )
        return cls(str(rede))

    @property
    def total_enderecos(self) -> int:
        return ipaddress.ip_network(self.cidr).num_addresses


@dataclass
class PortaServico:
    numero: int
    protocolo: str = "tcp"
    estado: str = "open"
    servico: str = ""
    versao: str = ""

    def esta_aberta(self) -> bool:
        return self.estado == "open"


@dataclass
class Capacidade:
    tipo: TipoCapacidade
    status: StatusCapacidade = StatusCapacidade.DESCONHECIDA
    verificada_em: datetime | None = None

    def esta_disponivel(self) -> bool:
        return self.status is StatusCapacidade.DISPONIVEL


@dataclass
class Host:
    ip: str
    hostname: str = ""
    porta_ssh: int = 22
    portas: list[PortaServico] = field(default_factory=list)
    capacidades: list[Capacidade] = field(default_factory=list)
    tags: set[str] = field(default_factory=set)
    primeira_descoberta: datetime | None = None
    ultima_descoberta: datetime | None = None
    id: int | None = None

    # -- capacidades
    def status_capacidade(self, tipo: TipoCapacidade) -> StatusCapacidade:
        for cap in self.capacidades:
            if cap.tipo is tipo:
                return cap.status
        return StatusCapacidade.DESCONHECIDA

    def possui_capacidade(self, tipo: TipoCapacidade) -> bool:
        return self.status_capacidade(tipo) is StatusCapacidade.DISPONIVEL

    def definir_capacidade(
        self, tipo: TipoCapacidade, status: StatusCapacidade, quando: datetime | None = None
    ) -> None:
        quando = quando or agora()
        for cap in self.capacidades:
            if cap.tipo is tipo:
                cap.status, cap.verificada_em = status, quando
                return
        self.capacidades.append(Capacidade(tipo, status, quando))

    # -- dados
    def portas_abertas(self) -> list[PortaServico]:
        return [p for p in self.portas if p.esta_aberta()]

    def esta_desatualizado(self, limite: timedelta = timedelta(days=7), referencia: datetime | None = None) -> bool:
        if self.ultima_descoberta is None:
            return True
        return (referencia or agora()) - self.ultima_descoberta > limite

    def adicionar_tag(self, tag: str) -> None:
        self.tags.add(normalizar_tag(tag))

    def remover_tag(self, tag: str) -> None:
        self.tags.discard(normalizar_tag(tag))

    def mesclar_descoberta(self, novo: "Host", quando: datetime) -> None:
        """Atualiza dados voláteis preservando id, tags e primeira descoberta."""
        if novo.hostname:
            self.hostname = novo.hostname
        self.portas = list(novo.portas)
        self.porta_ssh = novo.porta_ssh
        self.ultima_descoberta = quando
        if self.primeira_descoberta is None:
            self.primeira_descoberta = quando


# ------------------------------------------------------------- execução/auditoria
@dataclass
class ResultadoExecucao:
    status: StatusExecucao
    stdout: str = ""
    stderr: str = ""
    exit_code: int | None = None
    duracao_s: float = 0.0

    @property
    def sucesso(self) -> bool:
        return self.status is StatusExecucao.CONCLUIDA


@dataclass
class FiltroHistorico:
    alvo: str | None = None
    tipo_acao: TipoAcao | None = None
    status: StatusExecucao | None = None
    de: datetime | None = None
    ate: datetime | None = None
    limite: int = 100


@dataclass
class Evidencia:
    """Registro de auditoria encadeado por hash (RF-09, RNF-06, RN-03)."""

    tipo_acao: TipoAcao
    descricao: str
    alvo: str  # IP do host ou CIDR da varredura
    risco: NivelRisco
    status: StatusExecucao
    detalhes: str = ""
    saida: str = ""
    erro: str = ""
    exit_code: int | None = None
    timestamp: datetime = field(default_factory=agora)
    id: int | None = None
    hash_anterior: str = ""
    hash_integridade: str = ""

    def calcular_hash(self) -> str:
        carga = {
            "timestamp": para_iso(self.timestamp),
            "tipo_acao": self.tipo_acao.value,
            "descricao": self.descricao,
            "alvo": self.alvo,
            "risco": self.risco.name,
            "status": self.status.value,
            "detalhes": self.detalhes,
            "saida": self.saida,
            "erro": self.erro,
            "exit_code": self.exit_code,
            "hash_anterior": self.hash_anterior,
        }
        bruto = json.dumps(carga, sort_keys=True, ensure_ascii=False, separators=(",", ":"))
        return hashlib.sha256(bruto.encode("utf-8")).hexdigest()

    def selar(self, hash_anterior: str) -> None:
        self.hash_anterior = hash_anterior
        self.hash_integridade = self.calcular_hash()

    def integra(self) -> bool:
        return self.hash_integridade == self.calcular_hash()
