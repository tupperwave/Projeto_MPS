from dataclasses import dataclass
from datetime import datetime


@dataclass(frozen=True, slots=True)
class Memento:
	id_anotacao: int
	host_ip: str
	conteudo: str
	criada_em: datetime
	atualizada_em: datetime



