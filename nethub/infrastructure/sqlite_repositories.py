"""Repositórios SQLite (implementam as portas HostRepository e EvidenceRepository)."""
from __future__ import annotations

import ipaddress
import json
import sqlite3
from typing import Iterator

from ..domain.enums import (
    NivelRisco, StatusCapacidade, StatusExecucao, TipoAcao, TipoCapacidade,
)
from ..domain.errors import FalhaPersistenciaUsuarios, UsuarioDuplicado
from ..domain.models import (
    Capacidade, Evidencia, FiltroHistorico, Host, PortaServico, de_iso, para_iso,
)
from ..domain.ports import EvidenceRepository, HostRepository, UserRepository
from ..domain.users import Usuario
from .sqlite_db import SqliteDatabase


def _opt_iso(dt) -> str | None:
    return para_iso(dt) if dt else None


def _opt_dt(texto: str | None):
    return de_iso(texto) if texto else None


class SqliteHostRepository(HostRepository):
    def __init__(self, banco: SqliteDatabase) -> None:
        self._db = banco

    def salvar(self, host: Host) -> Host:
        portas = [
            {"numero": p.numero, "protocolo": p.protocolo, "estado": p.estado, "servico": p.servico, "versao": p.versao}
            for p in host.portas
        ]
        caps = [
            {"tipo": c.tipo.value, "status": c.status.value, "verificada_em": _opt_iso(c.verificada_em)}
            for c in host.capacidades
        ]
        self._db.executar(
            """
            INSERT INTO hosts (ip, hostname, porta_ssh, portas_json, capacidades_json, tags_json,
                               primeira_descoberta, ultima_descoberta)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(ip) DO UPDATE SET
                hostname=excluded.hostname, porta_ssh=excluded.porta_ssh,
                portas_json=excluded.portas_json, capacidades_json=excluded.capacidades_json,
                tags_json=excluded.tags_json, primeira_descoberta=excluded.primeira_descoberta,
                ultima_descoberta=excluded.ultima_descoberta
            """,
            (
                host.ip, host.hostname, host.porta_ssh, json.dumps(portas), json.dumps(caps),
                json.dumps(sorted(host.tags)), _opt_iso(host.primeira_descoberta),
                _opt_iso(host.ultima_descoberta),
            ),
        )
        host.id = self._db.consultar("SELECT id FROM hosts WHERE ip = ?", (host.ip,))[0]["id"]
        return host

    def buscar_por_ip(self, ip: str) -> Host | None:
        linhas = self._db.consultar("SELECT * FROM hosts WHERE ip = ?", (ip,))
        return self._para_host(linhas[0]) if linhas else None

    def listar(self, tag: str | None = None) -> list[Host]:
        hosts = [self._para_host(r) for r in self._db.consultar("SELECT * FROM hosts")]
        if tag:
            hosts = [h for h in hosts if tag in h.tags]
        def chave(h: Host):
            ip = ipaddress.ip_address(h.ip)
            return (ip.version, int(ip))
        return sorted(hosts, key=chave)

    @staticmethod
    def _para_host(r: sqlite3.Row) -> Host:
        return Host(
            id=r["id"],
            ip=r["ip"],
            hostname=r["hostname"],
            porta_ssh=r["porta_ssh"],
            portas=[PortaServico(**p) for p in json.loads(r["portas_json"])],
            capacidades=[
                Capacidade(TipoCapacidade(c["tipo"]), StatusCapacidade(c["status"]), _opt_dt(c["verificada_em"]))
                for c in json.loads(r["capacidades_json"])
            ],
            tags=set(json.loads(r["tags_json"])),
            primeira_descoberta=_opt_dt(r["primeira_descoberta"]),
            ultima_descoberta=_opt_dt(r["ultima_descoberta"]),
        )


class SqliteEvidenceRepository(EvidenceRepository):
    def __init__(self, banco: SqliteDatabase) -> None:
        self._db = banco

    def salvar(self, ev: Evidencia) -> Evidencia:
        cur = self._db.executar(
            """
            INSERT INTO evidencias (timestamp, tipo_acao, descricao, alvo, risco, status, detalhes,
                                    saida, erro, exit_code, hash_anterior, hash_integridade)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                para_iso(ev.timestamp), ev.tipo_acao.value, ev.descricao, ev.alvo, ev.risco.name,
                ev.status.value, ev.detalhes, ev.saida, ev.erro, ev.exit_code,
                ev.hash_anterior, ev.hash_integridade,
            ),
        )
        ev.id = cur.lastrowid
        return ev

    def ultimo_hash(self) -> str:
        linhas = self._db.consultar("SELECT hash_integridade FROM evidencias ORDER BY id DESC LIMIT 1")
        return linhas[0]["hash_integridade"] if linhas else ""

    def buscar(self, filtro: FiltroHistorico) -> list[Evidencia]:
        where, params = [], []
        if filtro.alvo:
            where.append("alvo = ?"); params.append(filtro.alvo)
        if filtro.tipo_acao:
            where.append("tipo_acao = ?"); params.append(filtro.tipo_acao.value)
        if filtro.status:
            where.append("status = ?"); params.append(filtro.status.value)
        if filtro.de:
            where.append("timestamp >= ?"); params.append(para_iso(filtro.de))
        if filtro.ate:
            where.append("timestamp <= ?"); params.append(para_iso(filtro.ate))
        sql = "SELECT * FROM evidencias"
        if where:
            sql += " WHERE " + " AND ".join(where)
        sql += " ORDER BY id DESC LIMIT ?"
        params.append(max(1, filtro.limite))
        return [self._para_evidencia(r) for r in self._db.consultar(sql, params)]

    def buscar_por_id(self, id_: int) -> Evidencia | None:
        linhas = self._db.consultar("SELECT * FROM evidencias WHERE id = ?", (id_,))
        return self._para_evidencia(linhas[0]) if linhas else None

    def todas_em_ordem(self) -> Iterator[Evidencia]:
        for r in self._db.consultar("SELECT * FROM evidencias ORDER BY id ASC"):
            yield self._para_evidencia(r)

    @staticmethod
    def _para_evidencia(r: sqlite3.Row) -> Evidencia:
        return Evidencia(
            id=r["id"],
            timestamp=de_iso(r["timestamp"]),
            tipo_acao=TipoAcao(r["tipo_acao"]),
            descricao=r["descricao"],
            alvo=r["alvo"],
            risco=NivelRisco[r["risco"]],
            status=StatusExecucao(r["status"]),
            detalhes=r["detalhes"],
            saida=r["saida"],
            erro=r["erro"],
            exit_code=r["exit_code"],
            hash_anterior=r["hash_anterior"],
            hash_integridade=r["hash_integridade"],
        )


class SqliteUserRepository(UserRepository):
    """Persiste usuários no SQLite sem armazenar a senha original."""

    def __init__(self, banco: SqliteDatabase) -> None:
        self._db = banco

    def salvar(self, usuario: Usuario) -> Usuario:
        try:
            self._db.executar(
                "INSERT INTO usuarios (login, sal_senha, hash_senha) VALUES (?, ?, ?)",
                (usuario.login, usuario._sal, usuario._hash_senha),
            )
            return usuario
        except sqlite3.IntegrityError as exc:
            raise UsuarioDuplicado(f"Já existe um usuário com o login {usuario.login!r}.") from exc
        except sqlite3.Error as exc:
            raise FalhaPersistenciaUsuarios("Não foi possível salvar o usuário no SQLite.") from exc

    def buscar_por_login(self, login: str) -> Usuario | None:
        try:
            linhas = self._db.consultar("SELECT login, sal_senha, hash_senha FROM usuarios WHERE login = ?", (login,))
        except sqlite3.Error as exc:
            raise FalhaPersistenciaUsuarios("Não foi possível consultar o usuário no SQLite.") from exc
        return self._para_usuario(linhas[0]) if linhas else None

    def listar(self) -> list[Usuario]:
        try:
            linhas = self._db.consultar("SELECT login, sal_senha, hash_senha FROM usuarios ORDER BY login")
        except sqlite3.Error as exc:
            raise FalhaPersistenciaUsuarios("Não foi possível listar usuários no SQLite.") from exc
        return [self._para_usuario(linha) for linha in linhas]

    @staticmethod
    def _para_usuario(linha: sqlite3.Row) -> Usuario:
        try:
            return Usuario.restaurar(linha["login"], linha["sal_senha"], linha["hash_senha"])
        except (KeyError, TypeError, ValueError) as exc:
            raise FalhaPersistenciaUsuarios("Os dados de usuário no SQLite estão inválidos.") from exc
