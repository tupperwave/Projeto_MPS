import unittest
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

class TestRepositoryFactory(unittest.TestCase):
	def setUp(self) -> None:
		self.banco = Mock(spec=SqliteDatabase)

	def test_criar_monta_a_familia_de_repositorios_com_usuarios_em_memoria(
		self,
	) -> None:
		repositorios = RepositoryFactory(self.banco).criar()

		self.assertIsInstance(repositorios.hosts, SqliteHostRepository)
		self.assertIsInstance(repositorios.evidencias, SqliteEvidenceRepository)
		self.assertIsInstance(repositorios.usuarios, MemoryUserRepository)
		self.assertIsInstance(
			repositorios.anotacoes,
			SqliteHostAnnotationRepository,
		)

	def test_criar_monta_repositorio_de_usuarios_sqlite_quando_configurado(
		self,
	) -> None:
		repositorios = RepositoryFactory(
			self.banco,
			persistencia_usuarios="sqlite",
		).criar()

		self.assertIsInstance(repositorios.usuarios, SqliteUserRepository)

	def test_criar_rejeita_persistencia_de_usuarios_desconhecida(self) -> None:
		fabrica = RepositoryFactory(self.banco, persistencia_usuarios="invalida")

		with self.assertRaisesRegex(
			ValueError,
			r"Persistência de usuários deve ser 'memoria' ou 'sqlite'\.",
		):
			fabrica.criar()
