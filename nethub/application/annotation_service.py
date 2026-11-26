"""Casos de uso para manter anotações de hosts."""
from __future__ import annotations

from ..domain.annotations import AnotacaoHost, MementoAnotacao
from ..domain.errors import RegistroNaoEncontrado, SemAtualizacaoParaDesfazer
from ..domain.ports import (
    AnnotationMementoRepository,
    HostRepository,
    HostAnnotationRepository,
)


class HostAnnotationService:
    def __init__(
        self,
        repositorio: HostAnnotationRepository,
        hosts: HostRepository,
        mementos: AnnotationMementoRepository,
    ) -> None:
        self._repositorio = repositorio
        self._hosts = hosts
        self._mementos = mementos

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
        memento = anotacao.criar_memento()
        memento_anterior = self._mementos.obter_ultimo()
        anotacao.atualizar(conteudo)
        try:
            atualizada = self._repositorio.salvar(anotacao)
        except Exception:
            anotacao.restaurar(memento)
            raise
        try:
            self._mementos.salvar(memento)
        except Exception:
            anotacao.restaurar(memento)
            self._repositorio.salvar(anotacao)
            if memento_anterior is None:
                self._mementos.limpar()
            else:
                self._mementos.salvar(memento_anterior)
            raise
        return atualizada

    def desfazer_ultima_atualizacao(self) -> AnotacaoHost:
        memento = self._mementos.obter_ultimo()
        if memento is None:
            raise SemAtualizacaoParaDesfazer("Não há atualização de anotação para desfazer.")

        anotacao = self.obter(memento.id)
        estado_atual = anotacao.criar_memento()
        anotacao.restaurar(memento)
        try:
            restaurada = self._repositorio.salvar(anotacao)
        except Exception:
            anotacao.restaurar(estado_atual)
            raise
        self._mementos.limpar()
        return restaurada

    def excluir(self, id_: int) -> bool:
        if self._repositorio.buscar_por_id(id_) is None:
            return False
        excluida = self._repositorio.excluir(id_)
        memento = self._mementos.obter_ultimo()
        if excluida and memento is not None and memento.id == id_:
            self._mementos.limpar()
        return excluida
