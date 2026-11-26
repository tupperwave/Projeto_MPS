"""Commands usados pela fachada para executar casos de uso de anotações."""
from __future__ import annotations

from dataclasses import dataclass
from typing import Protocol, TypeVar

from ..domain.annotations import AnotacaoHost
from .annotation_service import HostAnnotationService

T = TypeVar("T")


class Command(Protocol[T]):
    """Contrato de um pedido executável pela fachada."""

    def execute(self) -> T: ...


class CommandInvoker:
    """Executor que desacopla a fachada dos serviços chamados pelos comandos."""

    def invoke(self, command: Command[T]) -> T:
        return command.execute()


@dataclass(frozen=True)
class CreateAnnotationCommand:
    service: HostAnnotationService
    host_ip: str
    content: str

    def execute(self) -> AnotacaoHost:
        return self.service.criar(self.host_ip, self.content)


@dataclass(frozen=True)
class GetAnnotationCommand:
    service: HostAnnotationService
    annotation_id: int

    def execute(self) -> AnotacaoHost:
        return self.service.obter(self.annotation_id)


@dataclass(frozen=True)
class ListAnnotationsCommand:
    service: HostAnnotationService
    host_ip: str | None = None

    def execute(self) -> list[AnotacaoHost]:
        return self.service.listar(self.host_ip)


@dataclass(frozen=True)
class UpdateAnnotationCommand:
    service: HostAnnotationService
    annotation_id: int
    content: str

    def execute(self) -> AnotacaoHost:
        return self.service.atualizar(self.annotation_id, self.content)


@dataclass(frozen=True)
class DeleteAnnotationCommand:
    service: HostAnnotationService
    annotation_id: int

    def execute(self) -> bool:
        return self.service.excluir(self.annotation_id)


@dataclass(frozen=True)
class UndoLastAnnotationUpdateCommand:
    service: HostAnnotationService

    def execute(self) -> AnotacaoHost:
        return self.service.desfazer_ultima_atualizacao()
