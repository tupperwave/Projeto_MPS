"""Fachada singleton de escopo da composição da aplicação."""
from __future__ import annotations

class FacadeSingletonController:
    """Disponibiliza os gerentes por uma instância única para cada composição."""

    _instances: dict[tuple[int, ...], "FacadeSingletonController"] = {}

    def __new__(cls, inventario, descoberta, menu, executor, evidencias, historico, usuarios, anotacoes):
        chave = tuple(id(item) for item in (inventario, descoberta, menu, executor, evidencias, historico, usuarios, anotacoes))
        if chave not in cls._instances:
            instancia = super().__new__(cls)
            instancia._chave_singleton = chave
            cls._instances[chave] = instancia
        return cls._instances[chave]

    def __init__(self, inventario, descoberta, menu, executor, evidencias, historico, usuarios, anotacoes) -> None:
        if getattr(self, "_inicializada", False):
            return
        self.inventario = inventario
        self.descoberta = descoberta
        self.menu = menu
        self.executor = executor
        self.evidencias = evidencias
        self.historico = historico
        self.usuarios = usuarios
        self.anotacoes = anotacoes
        self._inicializada = True

    def quantidade_entidades_cadastradas(self) -> int:
        """Conta hosts, usuários e anotações registrados na aplicação."""
        return (
            len(self.inventario.listar())
            + len(self.usuarios.listar())
            + len(self.anotacoes.listar())
        )
