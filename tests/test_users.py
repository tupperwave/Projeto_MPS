import io
import sqlite3
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest.mock import patch

from nethub.application.user_service import UserService
from nethub.bootstrap import criar_container
from nethub.domain.errors import (
    FalhaPersistenciaUsuarios,
    LoginInvalido,
    SenhaInvalida,
    UsuarioDuplicado,
)
from nethub.domain.users import Usuario
from nethub.infrastructure.memory_user_repository import MemoryUserRepository
from nethub.infrastructure.sqlite_db import SqliteDatabase
from nethub.infrastructure.sqlite_repositories import SqliteUserRepository
from nethub.presentation.cli import main
from nethub.presentation.tui import TelaUsuarios, Terminal

SENHA = "ValidPass1!"


class TestUsuario(unittest.TestCase):
    def test_valida_login(self):
        for login in ("", "   ", "usuario123", "logincommaisde12"):
            with self.subTest(login=login), self.assertRaises(LoginInvalido):
                Usuario(login, SENHA)

    def test_valida_senha(self):
        for senha in ("Ab1!", "lowercaseonly"):
            with self.subTest(senha=senha), self.assertRaises(SenhaInvalida):
                Usuario("ana", senha)
        with self.assertRaises(SenhaInvalida):
            Usuario("Abc!defG", "Abc!defG")

    def test_hash_verificacao_e_representacao(self):
        usuario = Usuario("ana", SENHA)
        self.assertTrue(usuario.verificar_senha(SENHA))
        self.assertFalse(usuario.verificar_senha("OutraSenha1!"))
        self.assertNotIn(SENHA, repr(usuario))
        self.assertNotIn(usuario._hash_senha.hex(), repr(usuario))
        self.assertNotIn("senha", repr(usuario).lower())


class TestUserService(unittest.TestCase):
    def setUp(self):
        self.servico = UserService(MemoryUserRepository())

    def test_cadastra_e_lista_sem_duplicar(self):
        cadastrado = self.servico.cadastrar("ana", SENHA)
        self.assertEqual([cadastrado.login], [u.login for u in self.servico.listar()])
        with self.assertRaises(UsuarioDuplicado):
            self.servico.cadastrar("ana", SENHA)


class TestSqliteUserRepository(unittest.TestCase):
    def test_persiste_e_reabre_sem_senha_em_texto_puro(self):
        with tempfile.TemporaryDirectory() as pasta:
            caminho = Path(pasta) / "usuarios.db"
            banco = SqliteDatabase(caminho)
            repo = SqliteUserRepository(banco)
            repo.salvar(Usuario("ana", SENHA))
            banco.fechar()

            banco = SqliteDatabase(caminho)
            repo = SqliteUserRepository(banco)
            usuario = repo.buscar_por_login("ana")
            self.assertIsNotNone(usuario)
            self.assertTrue(usuario.verificar_senha(SENHA))
            self.assertEqual(["ana"], [u.login for u in repo.listar()])
            self.assertIsNone(repo.buscar_por_login("inexistente"))
            self.assertNotIn(SENHA.encode(), caminho.read_bytes())
            banco.fechar()

    def test_login_duplicado(self):
        banco = SqliteDatabase(":memory:")
        repo = SqliteUserRepository(banco)
        repo.salvar(Usuario("ana", SENHA))
        with self.assertRaises(UsuarioDuplicado):
            repo.salvar(Usuario("ana", "Different1!"))
        banco.fechar()

    def test_converte_falhas_sqlite(self):
        class BancoFalho:
            def executar(self, *_args, **_kwargs):
                raise sqlite3.OperationalError("detalhe interno")

            def consultar(self, *_args, **_kwargs):
                raise sqlite3.OperationalError("detalhe interno")

        repo = SqliteUserRepository(BancoFalho())
        usuario = Usuario("ana", SENHA)
        with self.assertRaises(FalhaPersistenciaUsuarios):
            repo.salvar(usuario)
        with self.assertRaises(FalhaPersistenciaUsuarios):
            repo.buscar_por_login("ana")
        with self.assertRaises(FalhaPersistenciaUsuarios):
            repo.listar()


class TestSelecaoPersistenciaUsuarios(unittest.TestCase):
    def test_sqlite_persiste_e_memoria_nao_persiste_entre_containers(self):
        with tempfile.TemporaryDirectory() as pasta:
            caminho = Path(pasta) / "nethub.db"
            c = criar_container(caminho, persistencia_usuarios="sqlite")
            c.usuarios.cadastrar("ana", SENHA)
            c.banco.fechar()

            c = criar_container(caminho, persistencia_usuarios="sqlite")
            self.assertEqual(["ana"], [u.login for u in c.usuarios.listar()])
            c.banco.fechar()

            c = criar_container(caminho, persistencia_usuarios="memoria")
            self.assertEqual([], c.usuarios.listar())
            c.banco.fechar()


class TestCliUsuarios(unittest.TestCase):
    def rodar(self, container, *argv):
        saida, erro = io.StringIO(), io.StringIO()
        with redirect_stdout(saida), redirect_stderr(erro):
            codigo = main(list(argv), container=container)
        return codigo, saida.getvalue(), erro.getvalue()

    def test_cadastro_e_listagem_nao_exibem_credenciais(self):
        c = criar_container(":memory:")
        with patch("nethub.presentation.cli.getpass.getpass", return_value=SENHA):
            codigo, saida, _ = self.rodar(c, "usuarios", "cadastrar", "ana")
        self.assertEqual(codigo, 0)
        self.assertIn("cadastrado com sucesso", saida)
        codigo, saida, _ = self.rodar(c, "usuarios", "listar")
        self.assertEqual(codigo, 0)
        self.assertIn("ana", saida)
        self.assertNotIn(SENHA, saida)
        self.assertNotIn(c.usuarios.listar()[0]._hash_senha.hex(), saida)
        c.banco.fechar()

    def test_lista_vazia_e_erro_de_validacao(self):
        c = criar_container(":memory:")
        codigo, saida, _ = self.rodar(c, "usuarios", "listar")
        self.assertEqual(codigo, 0)
        self.assertIn("Nenhum usuário", saida)
        with patch("nethub.presentation.cli.getpass.getpass", return_value="fraca"):
            codigo, _, erro = self.rodar(c, "usuarios", "cadastrar", "ana")
        self.assertEqual(codigo, 2)
        self.assertIn("Erro:", erro)
        self.assertNotIn("fraca", erro)
        c.banco.fechar()


class TestTelaUsuarios(unittest.TestCase):
    def test_lista_e_cadastro_com_senha_mascarada(self):
        class TerminalFake:
            def __init__(self):
                self.prompts = []

            def pedir_texto(self, prompt, linhas_acima=None, *, ocultar=False):
                self.prompts.append((prompt, ocultar))
                return "ana" if len(self.prompts) == 1 else SENHA

        class AppFake:
            def __init__(self):
                self.c = criar_container(":memory:")
                self.t = TerminalFake()
                self.status = ""

        app = AppFake()
        tela = TelaUsuarios(app)
        self.assertIn("nenhum usuário", tela.itens()[0])
        self.assertTrue(tela.tratar_tecla("c", app))
        self.assertEqual(tela.itens(), ["ana"])
        self.assertEqual(app.t.prompts[1][1], True)
        self.assertNotIn(SENHA, "\n".join(tela.itens()))
        app.c.banco.fechar()

    def test_terminal_mascara_caracteres_da_senha(self):
        class TelaFake:
            def __init__(self):
                self.teclas = iter(list(SENHA) + ["\n"])
                self.textos = []

            def getmaxyx(self):
                return (24, 80)

            def keypad(self, _ativado):
                pass

            def erase(self):
                pass

            def refresh(self):
                pass

            def addstr(self, _y, _x, texto, _attr=0):
                self.textos.append(texto)

            def get_wch(self):
                return next(self.teclas)

        tela = TelaFake()
        terminal = Terminal(tela)
        self.assertEqual(terminal.pedir_texto("Senha: ", ocultar=True), SENHA)
        self.assertNotIn(SENHA, "".join(tela.textos))


if __name__ == "__main__":
    unittest.main()
