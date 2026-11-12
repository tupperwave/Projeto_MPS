"""Casos de uso de cadastro e listagem de usuários."""
from __future__ import annotations

from datetime import datetime, timezone

from ..domain.models import AcessoUsuario
from ..domain.ports import UserAccessRepository, UserRepository
from ..domain.users import Usuario


class UserService:
    def __init__(
        self,
        repositorio: UserRepository,
        repositorio_acessos: UserAccessRepository,
    ) -> None:
        self._repositorio = repositorio
        self._repositorio_acessos = repositorio_acessos

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
        return self._repositorio_acessos.registrar(acesso)