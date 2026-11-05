from dataclasses import dataclass
from enum import Enum


class MetodoAutenticacao(str, Enum):
	AGENTE = "agente"
	CHAVE_PRIVADA = "chave_privada"
	SENHA = "senha"


@dataclass(frozen=True)
class AcessoRemoto:
	"""Dados de acesso remoto sem armazenar a senha.

	``host_ip`` aceita tanto um endereço IP quanto um hostname DNS.
	"""

	host_ip: str
	usuario: str
	porta: int = 22
	metodo: MetodoAutenticacao = MetodoAutenticacao.AGENTE
	identidade: str | None = None
	referencia_segredo: str | None = None

	def __post_init__(self) -> None:
		if not isinstance(self.host_ip, str) or not self.host_ip.strip():
			raise ValueError("host_ip ou hostname não pode ser vazio")
		if not isinstance(self.usuario, str) or not self.usuario.strip():
			raise ValueError("usuário não pode ser vazio")
		if not 1 <= self.porta <= 65535:
			raise ValueError("porta deve estar entre 1 e 65535")

		if self.metodo is MetodoAutenticacao.CHAVE_PRIVADA:
			if not isinstance(self.identidade, str) or not self.identidade.strip():
				raise ValueError("identidade é obrigatória para chave privada")
		elif self.metodo is MetodoAutenticacao.SENHA:
			if not isinstance(self.referencia_segredo, str) or not self.referencia_segredo.strip():
				raise ValueError("referência ao segredo é obrigatória para senha")
		elif self.metodo is MetodoAutenticacao.AGENTE and self.identidade is not None:
			raise ValueError("identidade não é utilizada com agente")
