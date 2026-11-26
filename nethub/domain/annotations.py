"""Entidade de anotação associada a um host."""
from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime

from .memento import Memento
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

    def criar_memento(self) -> Memento:
        if self.id is None:
            raise ValueError("Não é possível criar um snapshot de uma anotação sem ID.")
        return Memento(
            host_ip=self.host_ip,
            conteudo=self.conteudo,
            id_anotacao=self.id,
            criada_em=self.criada_em,
            atualizada_em=self.atualizada_em,
        )

    def restaurar_memento(self, memento: Memento) -> None:
        if self.id != memento.id_anotacao:
            raise ValueError("O Memento pertence a outra anotação.")

        self.host_ip = memento.host_ip
        self.conteudo = memento.conteudo
        self.criada_em = memento.criada_em
        self.atualizada_em = memento.atualizada_em