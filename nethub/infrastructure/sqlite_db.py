"""Conexão SQLite e migrações (local-first, RN-07)."""
from __future__ import annotations

import os
import sqlite3
from pathlib import Path

_MIGRACOES = [
    # versão 1
    """
    CREATE TABLE hosts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        ip TEXT NOT NULL UNIQUE,
        hostname TEXT NOT NULL DEFAULT '',
        porta_ssh INTEGER NOT NULL DEFAULT 22,
        portas_json TEXT NOT NULL DEFAULT '[]',
        capacidades_json TEXT NOT NULL DEFAULT '[]',
        tags_json TEXT NOT NULL DEFAULT '[]',
        primeira_descoberta TEXT,
        ultima_descoberta TEXT
    );
    CREATE TABLE evidencias (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp TEXT NOT NULL,
        tipo_acao TEXT NOT NULL,
        descricao TEXT NOT NULL,
        alvo TEXT NOT NULL,
        risco TEXT NOT NULL,
        status TEXT NOT NULL,
        detalhes TEXT NOT NULL DEFAULT '',
        saida TEXT NOT NULL DEFAULT '',
        erro TEXT NOT NULL DEFAULT '',
        exit_code INTEGER,
        hash_anterior TEXT NOT NULL DEFAULT '',
        hash_integridade TEXT NOT NULL
    );
    CREATE INDEX idx_evidencias_alvo ON evidencias(alvo);
    CREATE INDEX idx_evidencias_timestamp ON evidencias(timestamp);
    """,
]


class SqliteDatabase:
    def __init__(self, caminho: str | Path = ":memory:") -> None:
        self._memoria = str(caminho) == ":memory:"
        if not self._memoria:
            caminho = Path(caminho).expanduser()
            caminho.parent.mkdir(parents=True, exist_ok=True)
        self._con = sqlite3.connect(str(caminho))
        self._con.row_factory = sqlite3.Row
        if not self._memoria:
            try:
                os.chmod(caminho, 0o600)  # evidências podem conter saída de comandos
            except OSError:
                pass
        self.migrar()

    def migrar(self) -> None:
        versao = self._con.execute("PRAGMA user_version").fetchone()[0]
        for i in range(versao, len(_MIGRACOES)):
            self._con.executescript(_MIGRACOES[i])
            self._con.execute(f"PRAGMA user_version = {i + 1}")
        self._con.commit()

    def executar(self, sql: str, params: tuple | list = ()) -> sqlite3.Cursor:
        cur = self._con.execute(sql, params)
        self._con.commit()
        return cur

    def consultar(self, sql: str, params: tuple | list = ()) -> list[sqlite3.Row]:
        return self._con.execute(sql, params).fetchall()

    def fechar(self) -> None:
        self._con.close()
