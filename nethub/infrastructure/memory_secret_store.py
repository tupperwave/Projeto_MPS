"""Armazenamento de segredos limitado ao tempo de vida do processo/sessão."""

from __future__ import annotations

from ..domain.ports import SecretStorePort


class MemorySecretStore(SecretStorePort):
	"""Guarda segredos em memória para ser injetado nos serviços da aplicação.

	O conteúdo nunca é impresso nem incluído na representação do objeto. Os
	valores são perdidos quando a instância é descartada ou a sessão termina.
	"""

	def __init__(self) -> None:
		self._secrets: dict[str, str] = {}

	def salvar(self, referencia: str, segredo: str) -> None:
		"""Cria ou substitui um segredo associado a ``reference``."""
		if not referencia or not isinstance(segredo, str):
		    raise ValueError("Referência e segredo são obrigatórios.")
		self._secrets[referencia] = segredo

	def obter(self, referencia: str) -> str | None:
		"""Retorna o segredo ou ``None`` quando a referência não existe."""
		return self._secrets.get(referencia)

	def excluir(self, referencia: str) -> bool:
		"""Exclui um segredo e informa se a referência existia."""
		return self._secrets.pop(referencia, None) is not None

	save = salvar
	get = obter
	delete = excluir

	def __repr__(self) -> str:
		return f"{type(self).__name__}()"
