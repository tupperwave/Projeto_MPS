"""Armazenamento de segredos limitado ao tempo de vida do processo/sessão."""

from __future__ import annotations

from typing import Any


class MemorySecretStore:
	"""Guarda segredos em memória para ser injetado nos serviços da aplicação.

	O conteúdo nunca é impresso nem incluído na representação do objeto. Os
	valores são perdidos quando a instância é descartada ou a sessão termina.
	"""

	def __init__(self) -> None:
		self._secrets: dict[str, Any] = {}

	def save(self, reference: str, secret: Any) -> None:
		"""Cria ou substitui um segredo associado a ``reference``."""
		self._secrets[reference] = secret

	def get(self, reference: str) -> Any | None:
		"""Retorna o segredo ou ``None`` quando a referência não existe."""
		return self._secrets.get(reference)

	def delete(self, reference: str) -> bool:
		"""Exclui um segredo e informa se a referência existia."""
		return self._secrets.pop(reference, None) is not None

	# Nomes explícitos para uso por serviços que preferem a nomenclatura de
	# segredo, mantendo a mesma implementação e sem expor o armazenamento.
	set_secret = save
	get_secret = get
	delete_secret = delete

	def __repr__(self) -> str:
		return f"{type(self).__name__}()"
