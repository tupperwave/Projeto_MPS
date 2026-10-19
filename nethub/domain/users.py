"""Entidade e regras de validação para usuários do NetHub."""
from __future__ import annotations

import hashlib
import hmac
import secrets
from dataclasses import dataclass, field

from .errors import LoginInvalido, SenhaInvalida

_TAMANHO_MIN_SENHA = 8
_TAMANHO_MAX_SENHA = 128
_TAMANHO_MAX_LOGIN = 12
_GRUPOS_ESPECIAIS = frozenset("!@#$%^&*()_+-=[]{}|'")
_SAL_BYTES = 16
_HASH_BYTES = 32
_SCRYPT_N = 1 << 17
_SCRYPT_R = 8
_SCRYPT_P = 1
_SCRYPT_MAXMEM = 256 * 1024 * 1024


@dataclass(frozen=True, init=False)
class Usuario:
    """Usuário validado que guarda somente sal e hash da senha."""

    login: str
    _sal: bytes = field(repr=False)
    _hash_senha: bytes = field(repr=False)

    def __init__(self, login: str, senha: str) -> None:
        login_validado = self._validar_login(login)
        self._validar_senha(senha, login_validado)
        sal = secrets.token_bytes(_SAL_BYTES)
        hash_senha = self._derivar_hash(senha, sal)

        object.__setattr__(self, "login", login_validado)
        object.__setattr__(self, "_sal", sal)
        object.__setattr__(self, "_hash_senha", hash_senha)

    @classmethod
    def restaurar(cls, login: str, sal: bytes, hash_senha: bytes) -> "Usuario":
        """Reconstrói a entidade a partir dos dados persistidos, sem a senha original."""
        login_validado = cls._validar_login(login)
        if not isinstance(sal, bytes) or len(sal) != _SAL_BYTES:
            raise ValueError("Sal de senha persistido inválido.")
        if not isinstance(hash_senha, bytes) or len(hash_senha) != _HASH_BYTES:
            raise ValueError("Hash de senha persistido inválido.")
        usuario = object.__new__(cls)
        object.__setattr__(usuario, "login", login_validado)
        object.__setattr__(usuario, "_sal", sal)
        object.__setattr__(usuario, "_hash_senha", hash_senha)
        return usuario

    @staticmethod
    def _validar_login(login: str) -> str:
        if not isinstance(login, str):
            raise LoginInvalido("O login deve ser um texto não vazio.")
        valor = login.strip()
        if not valor:
            raise LoginInvalido("O login não pode ser vazio.")
        if len(valor) > _TAMANHO_MAX_LOGIN:
            raise LoginInvalido(f"O login deve ter no máximo {_TAMANHO_MAX_LOGIN} caracteres.")
        if any(caractere.isdigit() for caractere in valor):
            raise LoginInvalido("O login não pode conter números.")
        return valor

    @staticmethod
    def _validar_senha(senha: str, login: str) -> None:
        if not isinstance(senha, str):
            raise SenhaInvalida("A senha deve ser um texto.")
        if not _TAMANHO_MIN_SENHA <= len(senha) <= _TAMANHO_MAX_SENHA:
            raise SenhaInvalida(
                f"A senha deve ter entre {_TAMANHO_MIN_SENHA} e {_TAMANHO_MAX_SENHA} caracteres."
            )

        grupos = (
            any("A" <= caractere <= "Z" for caractere in senha),
            any("a" <= caractere <= "z" for caractere in senha),
            any("0" <= caractere <= "9" for caractere in senha),
            any(caractere in _GRUPOS_ESPECIAIS for caractere in senha),
        )
        if sum(grupos) < 3:
            raise SenhaInvalida("A senha deve conter pelo menos três dos quatro grupos de caracteres.")
        if senha == login:
            raise SenhaInvalida("A senha deve ser diferente do login.")

    @staticmethod
    def _derivar_hash(senha: str, sal: bytes) -> bytes:
        return hashlib.scrypt(
            senha.encode("utf-8"),
            salt=sal,
            n=_SCRYPT_N,
            r=_SCRYPT_R,
            p=_SCRYPT_P,
            maxmem=_SCRYPT_MAXMEM,
            dklen=_HASH_BYTES,
        )

    def verificar_senha(self, senha: str) -> bool:
        """Compara a senha recebida com o hash sem expor o valor armazenado."""
        if not isinstance(senha, str):
            return False
        hash_recebido = self._derivar_hash(senha, self._sal)
        return hmac.compare_digest(self._hash_senha, hash_recebido)
