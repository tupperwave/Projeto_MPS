from typing import Optional

from ..domain.memento import Memento


class Caretaker:
	"""Mantém um único snapshot para desfazer uma atualização."""

	def __init__(self) -> None:
		self._memento: Optional[Memento] = None

	def guardar(self, memento: Memento) -> None:
		"""Guarda um snapshot, substituindo o anterior."""
		self._memento = memento

	def obter_ultimo(self) -> Optional[Memento]:
		"""Retorna o snapshot atual ou None se não houver snapshot."""
		return self._memento

	def descartar_ultimo(self) -> None:
		"""Descarta o snapshot atual após um desfazer bem-sucedido."""
		self._memento = None
