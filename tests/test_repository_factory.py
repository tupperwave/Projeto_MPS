from unittest.mock import Mock


from nethub.infrastructure.memory_user_repository import MemoryUserRepository
from nethub.infrastructure.repository_factory import RepositoryFactory
from nethub.infrastructure.sqlite_db import SqliteDatabase
from nethub.infrastructure.sqlite_repositories import (
	SqliteEvidenceRepository,
	SqliteHostAnnotationRepository,
	SqliteHostRepository,
	SqliteUserRepository,
)


def test_criar_monta_a_familia_de_repositorios_com_usuarios_em_memoria(
) -> None:
	banco = Mock(spec=SqliteDatabase)
	repositorios = RepositoryFactory(banco).criar()

	assert isinstance(repositorios.hosts, SqliteHostRepository)
	assert isinstance(repositorios.evidencias, SqliteEvidenceRepository)
	assert isinstance(repositorios.usuarios, MemoryUserRepository)
	assert isinstance(repositorios.anotacoes, SqliteHostAnnotationRepository)


def test_criar_monta_repositorio_de_usuarios_sqlite_quando_configurado(
) -> None:
	banco = Mock(spec=SqliteDatabase)
	repositorios = RepositoryFactory(
		banco,
		persistencia_usuarios="sqlite",
	).criar()

	assert isinstance(repositorios.usuarios, SqliteUserRepository)


def test_criar_rejeita_persistencia_de_usuarios_desconhecida(
) -> None:
	banco = Mock(spec=SqliteDatabase)
	fabrica = RepositoryFactory(banco, persistencia_usuarios="invalida")

	try:
		fabrica.criar()
	except ValueError as erro:
		assert str(erro) == "Persistência de usuários deve ser 'memoria' ou 'sqlite'."
	else:
		assert False, "RepositoryFactory deveria rejeitar persistência inválida"
