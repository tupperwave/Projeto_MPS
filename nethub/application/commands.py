from dataclasses import dataclass


class ComandoAnotacao:
	"""Contrato comum para os comandos dos casos de uso de anotações."""

	def execute(self, servico):
		raise NotImplementedError


@dataclass(frozen=True)
class CriarAnotacaoCommand(ComandoAnotacao):
	host_ip: str
	conteudo: str

	def execute(self, servico):
		return servico.criar(self.host_ip, self.conteudo)


@dataclass(frozen=True)
class ObterAnotacaoCommand(ComandoAnotacao):
	id_anotacao: int

	def execute(self, servico):
		return servico.obter(self.id_anotacao)


@dataclass(frozen=True)
class ListarAnotacoesCommand(ComandoAnotacao):
	host_ip: str | None = None

	def execute(self, servico):
		return servico.listar(self.host_ip)


@dataclass(frozen=True)
class AtualizarAnotacaoCommand(ComandoAnotacao):
	id_anotacao: int
	conteudo: str

	def execute(self, servico):
		return servico.atualizar(self.id_anotacao, self.conteudo)


@dataclass(frozen=True)
class ExcluirAnotacaoCommand(ComandoAnotacao):
	id_anotacao: int

	def execute(self, servico):
		return servico.excluir(self.id_anotacao)