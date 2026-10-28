"""Interface de linha de comando (scriptável) do NetHub."""
from __future__ import annotations

import argparse
import getpass
import shlex
import sys
from datetime import datetime, time as hora

from ..bootstrap import Container, criar_container
from ..domain.actions import ExecutarComandoAcao
from ..domain.enums import StatusExecucao, TipoAcao, TipoConfirmacao
from ..domain.errors import NetHubError
from ..domain.models import AlvoRede, FiltroHistorico
from ..domain.ports import ConfirmationPort, PedidoConfirmacao
from . import formatacao as fmt

COD_OK, COD_FALHA, COD_ERRO, COD_CANCELADO = 0, 1, 2, 3


class CliConfirmacao(ConfirmationPort):
    """Confirmação proporcional ao risco (RF-04). ALTO/CRÍTICO exigem digitar o IP do alvo."""

    def __init__(self, sim: bool = False, confirmar_ip: str | None = None, entrada=input) -> None:
        self._sim = sim
        self._ip = confirmar_ip
        self._entrada = entrada

    def confirmar(self, pedido: PedidoConfirmacao) -> bool:
        try:
            if pedido.tipo is TipoConfirmacao.SIMPLES:
                if self._sim:
                    return True
                return self._entrada(f"Risco {pedido.risco.rotulo}. Confirmar execução? [s/N] ").strip().lower() in ("s", "sim", "y")
            if self._ip is not None:
                return self._ip == pedido.host_ip
            resp = self._entrada(f"Risco {pedido.risco.rotulo}! Digite o IP do alvo ({pedido.host_ip}) para confirmar: ")
            return resp.strip() == pedido.host_ip
        except EOFError:
            return False


def _imprimir(linhas: list[str]) -> None:
    print("\n".join(linhas))


def _parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="nethub", description="NetHub - administração de infraestrutura pelo terminal (MVP).")
    p.add_argument("--db", help="caminho do banco SQLite (padrão: $NETHUB_DB ou ~/.nethub/nethub.db)")
    p.add_argument(
        "--persistencia-usuarios", choices=("memoria", "sqlite"), default="memoria",
        help="armazenamento de usuários (padrão: memoria)",
    )
    sub = p.add_subparsers(dest="cmd", required=True)

    d = sub.add_parser("descobrir", help="varre uma rede com Nmap e atualiza o inventário (UC-01)")
    d.add_argument("alvo", help="IP ou CIDR, ex.: 192.168.0.0/24")
    d.add_argument("--autorizado", action="store_true", help="confirmo que tenho autorização para varrer este alvo")

    h = sub.add_parser("hosts", help="lista os hosts do inventário")
    h.add_argument("--tag")

    one = sub.add_parser("host", help="detalhes, capacidades e ações de um host (UC-02)")
    one.add_argument("ip")

    e = sub.add_parser("exec", help="executa um comando remoto não interativo via SSH (UC-03)")
    e.add_argument("ip")
    e.add_argument("comando", nargs="*", help="comando remoto; prefira usar -- antes dele (ex.: exec IP --yes -- df -h)")
    e.add_argument("--usuario", help="usuário SSH")
    e.add_argument("--identidade", help="arquivo de chave privada (ssh -i)")
    e.add_argument("--timeout", type=float, default=30.0)
    e.add_argument("--simular", action="store_true", help="apenas mostra o plano (dry-run), sem executar")
    e.add_argument("--yes", action="store_true", help="confirma automaticamente ações de risco MÉDIO")
    e.add_argument("--confirmar-ip", help="confirma ações ALTO/CRÍTICO de forma não interativa (deve ser igual ao IP alvo)")

    hi = sub.add_parser("historico", help="consulta o histórico de operações (UC-08)")
    hi.add_argument("--host")
    hi.add_argument("--acao", choices=[t.value for t in TipoAcao])
    hi.add_argument("--resultado", choices=[s.value for s in StatusExecucao])
    hi.add_argument("--de", help="data inicial AAAA-MM-DD")
    hi.add_argument("--ate", help="data final AAAA-MM-DD")
    hi.add_argument("--limite", type=int, default=50)
    hi.add_argument("--detalhe", type=int, metavar="ID", help="mostra um registro completo")

    sub.add_parser("verificar-auditoria", help="verifica a integridade da cadeia de evidências")

    usuarios = sub.add_parser("usuarios", help="cadastra e lista usuários")
    usuarios_sub = usuarios.add_subparsers(dest="usuarios_cmd", required=True)
    cadastrar = usuarios_sub.add_parser("cadastrar", help="cadastra um usuário")
    cadastrar.add_argument("login")
    usuarios_sub.add_parser("listar", help="lista usuários cadastrados")

    t = sub.add_parser("tag", help="organiza hosts com tags (UC-09)")
    t.add_argument("operacao", choices=["add", "rm"])
    t.add_argument("ip")
    t.add_argument("nome")

    tui = sub.add_parser("tui", help="abre a interface de terminal interativa")
    tui.add_argument("--usuario")
    tui.add_argument("--identidade")
    return p


def _data(texto: str | None, fim_do_dia: bool = False) -> datetime | None:
    if not texto:
        return None
    try:
        dia = datetime.strptime(texto, "%Y-%m-%d")
    except ValueError as exc:
        raise NetHubError(f"Data inválida (use AAAA-MM-DD): {texto!r}") from exc
    return datetime.combine(dia.date(), hora.max if fim_do_dia else hora.min).astimezone()


def main(argv: list[str] | None = None, container: Container | None = None, entrada=input) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    restante: list[str] = []
    if "--" in argv:  # tudo após '--' é o comando remoto, intocado pelo argparse
        i = argv.index("--")
        argv, restante = argv[:i], argv[i + 1:]
    args = _parser().parse_args(argv)
    if args.cmd == "exec":
        args.comando = restante or args.comando
    c = container
    criado_aqui = c is None
    try:
        if c is None:
            c = criar_container(
                args.db,
                getattr(args, "usuario", None),
                getattr(args, "identidade", None),
                persistencia_usuarios=args.persistencia_usuarios,
            )
        return _despachar(args, c, entrada)
    except NetHubError as exc:
        print(f"Erro: {exc}", file=sys.stderr)
        return COD_ERRO
    except ValueError as exc:  # ex.: usuário SSH inválido
        print(f"Erro: {exc}", file=sys.stderr)
        return COD_ERRO
    finally:
        if criado_aqui and c is not None:
            c.fechar()


def _despachar(args, c: Container, entrada) -> int:
    if args.cmd == "usuarios":
        if args.usuarios_cmd == "cadastrar":
            senha = getpass.getpass("Senha: ")
            c.usuarios.cadastrar(args.login, senha)
            print(f"Usuário {args.login!r} cadastrado com sucesso.")
            return COD_OK
        usuarios = c.usuarios.listar()
        if not usuarios:
            print("Nenhum usuário cadastrado.")
            return COD_OK
        print("Usuários cadastrados:")
        for usuario in usuarios:
            print(f"- {usuario.login}")
        return COD_OK

    if args.cmd == "descobrir":
        alvo = AlvoRede.criar(args.alvo)
        autorizado = args.autorizado
        if not autorizado:
            autorizado = entrada(f"Você tem autorização para varrer {alvo.cidr} ({alvo.total_enderecos} endereços)? [s/N] ").strip().lower() in ("s", "sim")
        r = c.descoberta.descobrir(alvo, autorizado)
        if not r.hosts:
            print("Varredura concluída: nenhum host encontrado.")
            return COD_OK
        _imprimir(fmt.tabela(fmt.CABECALHO_HOSTS, [fmt.resumo_host(h) for h in r.hosts]))
        print(f"\n{len(r.hosts)} host(s): {r.novos} novo(s), {r.atualizados} atualizado(s).")
        return COD_OK

    if args.cmd == "hosts":
        hosts = c.inventario.listar(args.tag)
        if not hosts:
            print("Inventário vazio. Use: nethub descobrir <CIDR>")
            return COD_OK
        _imprimir(fmt.tabela(fmt.CABECALHO_HOSTS, [fmt.resumo_host(h) for h in hosts]))
        return COD_OK

    if args.cmd == "host":
        host = c.inventario.obter(args.ip)
        _imprimir(fmt.linhas_host(host, c.menu.acoes_compativeis(host)))
        return COD_OK

    if args.cmd == "exec":
        comando = args.comando
        if not comando:
            raise NetHubError("Informe o comando após '--'. Ex.: nethub exec 10.0.0.5 -- uptime")
        texto = comando[0] if len(comando) == 1 else shlex.join(comando)
        host = c.inventario.obter(args.ip)
        acao = ExecutarComandoAcao(texto, timeout_s=args.timeout)
        plano = c.executor.simular(acao, host)
        _imprimir(fmt.linhas_plano(plano))
        if args.simular:
            return COD_OK
        print()
        r = c.executor.executar(acao, host, CliConfirmacao(args.yes, args.confirmar_ip, entrada), dry_run_realizado=True)
        _imprimir(fmt.linhas_resultado(r))
        if r.resultado.status is StatusExecucao.CANCELADA:
            return COD_CANCELADO
        return COD_OK if r.resultado.sucesso else COD_FALHA

    if args.cmd == "historico":
        if args.detalhe is not None:
            _imprimir(fmt.linhas_evidencia(c.historico.detalhar(args.detalhe)))
            return COD_OK
        filtro = FiltroHistorico(
            alvo=args.host,
            tipo_acao=TipoAcao(args.acao) if args.acao else None,
            status=StatusExecucao(args.resultado) if args.resultado else None,
            de=_data(args.de), ate=_data(args.ate, fim_do_dia=True), limite=args.limite,
        )
        registros = c.historico.consultar(filtro)
        if not registros:
            print("Nenhum registro encontrado para os filtros informados.")
            return COD_OK
        _imprimir(fmt.tabela(fmt.CABECALHO_HISTORICO, [fmt.linha_historico(e) for e in registros]))
        return COD_OK

    if args.cmd == "verificar-auditoria":
        ok, quebrado = c.evidencias.verificar_cadeia()
        if ok:
            print("Cadeia de evidências íntegra.")
            return COD_OK
        print(f"ALERTA: inconsistência detectada a partir do registro #{quebrado}.", file=sys.stderr)
        return COD_FALHA

    if args.cmd == "tag":
        host = c.inventario.adicionar_tag(args.ip, args.nome) if args.operacao == "add" else c.inventario.remover_tag(args.ip, args.nome)
        print(f"{host.ip}: tags = {', '.join(sorted(host.tags)) or '-'}")
        return COD_OK

    if args.cmd == "tui":
        from .tui import executar_tui
        executar_tui(c)
        return COD_OK

    return COD_ERRO


if __name__ == "__main__":
    raise SystemExit(main())
