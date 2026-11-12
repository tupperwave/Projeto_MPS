"""Casos de uso de cadastro e listagem de usuários."""
from __future__ import annotations

from datetime import datetime, timezone

from ..domain.models import AcessoUsuario
from ..domain.ports import UserAccessRepository, UserRepository,  LoggerPort
from ..domain.users import Usuario


class UserService:
    def __init__(
        self,
        repositorio: UserRepository,
        repositorio_acessos: UserAccessRepository,
        logger: LoggerPort,
    ) -> None:
        self._repositorio = repositorio
        self._repositorio_acessos = repositorio_acessos
        self._logger = logger

    def cadastrar(self, login: str, senha: str) -> Usuario:
        usuario = Usuario(login, senha)
        return self._repositorio.salvar(usuario)

    def listar(self) -> list[Usuario]:
        return self._repositorio.listar()

    def registrar_acesso(self, login: str, sucesso: bool) -> AcessoUsuario:
        """Registra uma tentativa de acesso e retorna o registro persistido."""
        acesso = AcessoUsuario(
            login=login,
            ocorrido_em=datetime.now(timezone.utc),
            sucesso=sucesso,
        )

        persistido = self._repositorio_acessos.registrar(acesso)

        self._logger.info(
            "Tentativa de acesso registrada",
            login=login,
            sucesso=sucesso,
        )

        return persistido