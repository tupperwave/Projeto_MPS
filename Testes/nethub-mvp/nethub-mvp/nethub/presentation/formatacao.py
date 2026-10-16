"""Formatação de texto compartilhada entre CLI e TUI."""
from __future__ import annotations

from datetime import datetime

from ..application.action_executor import PlanoSimulado, ResultadoAcao
from ..domain.actions import Acao
from ..domain.enums import StatusCapacidade, StatusExecucao, TipoCapacidade
from ..domain.models import Evidencia, Host

_STATUS_EXEC = {
    StatusExecucao.CONCLUIDA: "concluída",
    StatusExecucao.FALHA: "FALHA",
    StatusExecucao.CANCELADA: "cancelada",
    StatusExecucao.EXPIRADA: "EXPIRADA",
}
_STATUS_CAP = {
    StatusCapacidade.DISPONIVEL: "disponível",
    StatusCapacidade.DESCONHECIDA: "desconhecida (não verificada)",
    StatusCapacidade.INDISPONIVEL: "indisponível",
}


def rotulo_status(status: StatusExecucao) -> str:
    return _STATUS_EXEC[status]


def data_local(dt: datetime | None) -> str:
    return dt.astimezone().strftime("%Y-%m-%d %H:%M:%S") if dt else "nunca"


def tabela(cabecalho: list[str], linhas: list[list[str]]) -> list[str]:
    larguras = [len(c) for c in cabecalho]
    for linha in linhas:
        for i, cel in enumerate(linha):
            larguras[i] = max(larguras[i], len(cel))
    def fmt(cels: list[str]) -> str:
        return "  ".join(c.ljust(larguras[i]) for i, c in enumerate(cels)).rstrip()
    return [fmt(cabecalho), "  ".join("-" * w for w in larguras)] + [fmt(l) for l in linhas]


def resumo_host(h: Host) -> list[str]:
    portas = ",".join(str(p.numero) for p in h.portas_abertas()) or "-"
    ssh = "ssh" if h.possui_capacidade(TipoCapacidade.SSH) else "-"
    return [h.ip, h.hostname or "-", ssh, portas, ",".join(sorted(h.tags)) or "-", data_local(h.ultima_descoberta)]


CABECALHO_HOSTS = ["IP", "HOSTNAME", "SSH", "PORTAS ABERTAS", "TAGS", "ÚLTIMA DESCOBERTA"]


def linhas_host(h: Host, acoes: list[type[Acao]]) -> list[str]:
    out = [
        f"IP:                 {h.ip}",
        f"Hostname:           {h.hostname or '-'}",
        f"Porta SSH:          {h.porta_ssh}",
        f"Tags:               {', '.join(sorted(h.tags)) or '-'}",
        f"Primeira descoberta: {data_local(h.primeira_descoberta)}",
        f"Última descoberta:   {data_local(h.ultima_descoberta)}"
        + ("   [DESATUALIZADO: visto há mais de 7 dias]" if h.esta_desatualizado() else ""),
        "",
        "Capacidades:",
    ]
    for tipo in TipoCapacidade:
        out.append(f"  {tipo.value:<8} {_STATUS_CAP[h.status_capacidade(tipo)]}")
    out += ["", "Portas abertas / serviços:"]
    abertas = h.portas_abertas()
    if abertas:
        out += ["  " + l for l in tabela(["PORTA", "PROTO", "SERVIÇO", "VERSÃO"],
                [[str(p.numero), p.protocolo, p.servico or "-", p.versao or "-"] for p in abertas])]
    else:
        out.append("  (nenhuma porta aberta registrada)")
    out += ["", "Ações disponíveis:"]
    if acoes:
        out += [f"  [{i}] {a.nome}" for i, a in enumerate(acoes, 1)]
    else:
        out.append("  (nenhuma ação compatível: capacidades não verificadas ou indisponíveis)")
    return out


def linhas_plano(p: PlanoSimulado) -> list[str]:
    return [
        "DRY-RUN / PLANO DA OPERAÇÃO (nada foi executado ainda)",
        f"  Alvo:      {p.host_ip}",
        f"  Operação:  {p.descricao}",
        f"  Detalhe:   {p.detalhes}",
        f"  Risco:     {p.risco.rotulo}",
        f"  Confirmação exigida: {p.tipo_confirmacao.value}",
    ]


def linhas_evidencia(ev: Evidencia, completo: bool = True) -> list[str]:
    out = [
        f"Registro #{ev.id}",
        f"  Data:       {data_local(ev.timestamp)}",
        f"  Alvo:       {ev.alvo}",
        f"  Ação:       {ev.tipo_acao.value}",
        f"  Descrição:  {ev.descricao}",
        f"  Risco:      {ev.risco.rotulo}",
        f"  Resultado:  {rotulo_status(ev.status)}" + (f" (exit code {ev.exit_code})" if ev.exit_code is not None else ""),
    ]
    if completo:
        out += [f"  Hash:       {ev.hash_integridade[:16]}…  (anterior: {ev.hash_anterior[:16] or '-'}…)",
                f"  Íntegro:    {'sim' if ev.integra() else 'NÃO (registro adulterado)'}"]
        if ev.saida:
            out += ["", "--- saída ---"] + ev.saida.rstrip("\n").splitlines()
        if ev.erro:
            out += ["", "--- erro ---"] + ev.erro.rstrip("\n").splitlines()
    return out


def linhas_resultado(r: ResultadoAcao) -> list[str]:
    res = r.resultado
    out = [
        f"Resultado: {rotulo_status(res.status)}"
        + (f"   exit code: {res.exit_code}" if res.exit_code is not None else "")
        + (f"   duração: {res.duracao_s:.2f}s" if res.duracao_s else ""),
        f"Evidência registrada: #{r.evidencia.id}",
    ]
    if res.status is StatusExecucao.CANCELADA:
        out.append("Operação cancelada: nada foi executado.")
    if res.stdout:
        out += ["", "--- saída ---"] + res.stdout.rstrip("\n").splitlines()
    if res.stderr:
        out += ["", "--- erro ---"] + res.stderr.rstrip("\n").splitlines()
    return out


def linha_historico(ev: Evidencia) -> list[str]:
    return [str(ev.id), data_local(ev.timestamp), ev.alvo, ev.tipo_acao.value, ev.risco.rotulo, rotulo_status(ev.status)]


CABECALHO_HISTORICO = ["ID", "DATA", "ALVO", "AÇÃO", "RISCO", "RESULTADO"]
