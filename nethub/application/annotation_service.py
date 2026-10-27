"""Casos de uso para manter anotações de hosts."""
from __future__ import annotations

from ..domain.annotations import AnotacaoHost
from ..domain.errors import RegistroNaoEncontrado
from ..domain.ports import HostRepository, HostAnnotationRepository


class HostAnnotationService:
    def __init__(self, repositorio: HostAnnotationRepository, hosts: HostRepository) -> None:
        self._repositorio = repositorio
        self._hosts = hosts

    def criar(self, host_ip: str, conteudo: str) -> AnotacaoHost:
        if self._hosts.buscar_por_ip(host_ip) is None:
            raise RegistroNaoEncontrado(f"Host {host_ip} não encontrado.")
        return self._repositorio.salvar(AnotacaoHost(host_ip, conteudo))

    def obter(self, id_: int) -> AnotacaoHost:
        anotacao = self._repositorio.buscar_por_id(id_)
        if anotacao is None:
            raise RegistroNaoEncontrado(f"Anotação #{id_} não encontrada.")
        return anotacao

    def listar(self, host_ip: str | None = None) -> list[AnotacaoHost]:
        return self._repositorio.listar(host_ip)

    def atualizar(self, id_: int, conteudo: str) -> AnotacaoHost:
        anotacao = self.obter(id_)
        anotacao.atualizar(conteudo)
        return self._repositorio.salvar(anotacao)

    def excluir(self, id_: int) -> bool:
        if self._repositorio.buscar_por_id(id_) is None:
            return False
        return self._repositorio.excluir(id_)
