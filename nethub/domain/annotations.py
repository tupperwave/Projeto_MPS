"""Entidade de anotação associada a um host."""
from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime

from .models import agora


@dataclass
class AnotacaoHost:
    host_ip: str
    conteudo: str
    id: int | None = None
    criada_em: datetime = field(default_factory=agora)
    atualizada_em: datetime = field(default_factory=agora)

    def __post_init__(self) -> None:
        self.host_ip = (self.host_ip or "").strip()
        self.conteudo = (self.conteudo or "").strip()
        if not self.host_ip:
            raise ValueError("O endereço do host é obrigatório.")
        if not self.conteudo:
            raise ValueError("O conteúdo da anotação é obrigatório.")

    def atualizar(self, conteudo: str) -> None:
        conteudo = (conteudo or "").strip()
        if not conteudo:
            raise ValueError("O conteúdo da anotação é obrigatório.")
        self.conteudo = conteudo
        self.atualizada_em = agora()
