"""Serviço de aplicação para gerenciar acessos remotos."""

from dataclasses import dataclass, field


@dataclass(frozen=True)
class AcessoRemoto:
	host_ip: str
	usuario: str
	senha: str = field(repr=False)
	porta: int = 22


@dataclass(frozen=True)
class ContextoAcesso:
	host_ip: str
	usuario: str
	senha: str = field(repr=False)
	porta: int = 22


_acessos: dict[str, AcessoRemoto] = {}


def cadastrar(
	host_ip: str,
	usuario: str,
	senha: str,
	porta: int = 22,
) -> AcessoRemoto:
	"""Cadastra ou substitui as credenciais de um host."""
	if not host_ip.strip():
		raise ValueError("host_ip não pode ser vazio")
	if not usuario:
		raise ValueError("usuario não pode ser vazio")
	if not senha:
		raise ValueError("senha não pode ser vazia")
	if porta < 1 or porta > 65535:
		raise ValueError("porta deve estar entre 1 e 65535")

	acesso = AcessoRemoto(host_ip, usuario, senha, porta)
	_acessos[host_ip] = acesso
	return acesso


def obter(host_ip: str) -> AcessoRemoto | None:
	"""Obtém o acesso cadastrado para o host, se existir."""
	return _acessos.get(host_ip)


def remover(host_ip: str) -> bool:
	"""Remove o acesso do host e informa se ele estava cadastrado."""
	return _acessos.pop(host_ip, None) is not None


def resolver_credencial(host_ip: str) -> ContextoAcesso:
	"""Resolve as credenciais necessárias para acessar um host."""
	acesso = obter(host_ip)
	if acesso is None:
		raise LookupError(f"Nenhum acesso remoto cadastrado para {host_ip}")
	return ContextoAcesso(
		host_ip=acesso.host_ip,
		usuario=acesso.usuario,
		senha=acesso.senha,
		porta=acesso.porta,
	)