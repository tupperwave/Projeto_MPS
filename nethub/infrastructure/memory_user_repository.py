"""Repositório de usuários em coleção mantida na memória do processo."""
from __future__ import annotations

from ..domain.errors import UsuarioDuplicado
from ..domain.ports import UserRepository
from ..domain.users import Usuario


class MemoryUserRepository(UserRepository):
    def __init__(self) -> None:
        self._usuarios: dict[str, Usuario] = {}

    def salvar(self, usuario: Usuario) -> Usuario:
        if usuario.login in self._usuarios:
            raise UsuarioDuplicado(f"Já existe um usuário com o login {usuario.login!r}.")
        self._usuarios[usuario.login] = usuario
        return usuario

    def buscar_por_login(self, login: str) -> Usuario | None:
        return self._usuarios.get(login)

    def listar(self) -> list[Usuario]:
        return list(self._usuarios.values())

    def contar(self) -> int:
        return len(self._usuarios)
