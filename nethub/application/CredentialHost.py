from dataclasses import dataclass
from enum import Enum


class MetodoAutenticacao(str, Enum):
	AGENTE = "agente"
	CHAVE_PRIVADA = "chave_privada"
	SENHA = "senha"


@dataclass(frozen=True)
class AcessoRemoto:
	host_ip: str
	usuario: str
	porta: int = 22
	metodo: MetodoAutenticacao = MetodoAutenticacao.AGENTE
	identidade: str | None = None
	referencia_segredo: str | None = None
