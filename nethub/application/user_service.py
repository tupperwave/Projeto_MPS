"""Casos de uso de cadastro e listagem de usuários."""
from __future__ import annotations

from ..domain.ports import UserRepository
from ..domain.users import Usuario


class UserService:
    def __init__(self, repositorio: UserRepository) -> None:
        self._repositorio = repositorio

    def cadastrar(self, login: str, senha: str) -> Usuario:
        usuario = Usuario(login, senha)
        return self._repositorio.salvar(usuario)

    def listar(self) -> list[Usuario]:
        return self._repositorio.listar()
