"""Enumerações do domínio."""
from enum import Enum, IntEnum


class NivelRisco(IntEnum):
    """Níveis de risco (RF-04). IntEnum permite comparar e tomar o máximo."""

    BAIXO = 1
    MEDIO = 2
    ALTO = 3
    CRITICO = 4

    @property
    def rotulo(self) -> str:
        return _ROTULOS_RISCO[self]


_ROTULOS_RISCO = {
    NivelRisco.BAIXO: "Baixo",
    NivelRisco.MEDIO: "Médio",
    NivelRisco.ALTO: "Alto",
    NivelRisco.CRITICO: "Crítico",
}


class TipoConfirmacao(Enum):
    NENHUMA = "nenhuma"
    SIMPLES = "simples"
    EXPLICITA = "explicita"


class TipoCapacidade(str, Enum):
    SSH = "ssh"
    SFTP = "sftp"
    SYSTEMD = "systemd"
    SUDO = "sudo"


class StatusCapacidade(str, Enum):
    DISPONIVEL = "disponivel"
    DESCONHECIDA = "desconhecida"
    INDISPONIVEL = "indisponivel"


class StatusExecucao(str, Enum):
    CONCLUIDA = "concluida"
    FALHA = "falha"
    CANCELADA = "cancelada"
    EXPIRADA = "expirada"


class TipoAcao(str, Enum):
    DESCOBERTA = "descoberta"
    COMANDO_REMOTO = "comando_remoto"
    # Reservados para as próximas iterações (mantém o esquema do banco estável):
    SAUDE_HOST = "saude_host"
    REINICIAR_SERVICO = "reiniciar_servico"
    TRANSFERIR_ARQUIVO = "transferir_arquivo"
    WORKFLOW = "workflow"
