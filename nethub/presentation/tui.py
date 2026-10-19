"""TUI (curses, só biblioteca padrão). Telas = classes simples; o loop e o desenho ficam em NetHubTui/Terminal."""
from __future__ import annotations

import curses
import os
import sys

from ..bootstrap import Container
from ..domain.actions import ExecutarComandoAcao
from ..domain.enums import TipoConfirmacao
from ..domain.errors import AlvoInvalido, ComandoInvalido, NetHubError
from ..domain.models import AlvoRede, FiltroHistorico, Host
from ..domain.ports import ConfirmationPort, PedidoConfirmacao
from . import formatacao as fmt

ENTER = ("\n", "\r", curses.KEY_ENTER)
ESC = "\x1b"
BACKSPACE = ("\x7f", "\b", curses.KEY_BACKSPACE)


class Terminal:
    """Fina camada sobre curses: desenho, leitura de teclas e entrada de texto."""

    def __init__(self, tela) -> None:
        self._t = tela
        try:
            curses.curs_set(0)
        except curses.error:
            pass
        tela.keypad(True)

    def tamanho(self) -> tuple[int, int]:
        return self._t.getmaxyx()

    def tecla(self):
        try:
            return self._t.get_wch()
        except curses.error:
            return None

    def _escrever(self, y: int, x: int, texto: str, attr: int = 0) -> None:
        h, w = self.tamanho()
        if y < 0 or y >= h or x >= w - 1:
            return
        try:
            self._t.addstr(y, x, texto[: max(0, w - 1 - x)], attr)
        except curses.error:
            pass

    def desenhar(self, titulo, cabecalho, itens, indice, topo, selecionavel, rodape, status) -> None:
        h, w = self.tamanho()
        self._t.erase()
        self._escrever(0, 0, f" NetHub › {titulo} ".ljust(w - 1), curses.A_REVERSE)
        y0 = 1
        if cabecalho:
            self._escrever(1, 0, cabecalho, curses.A_BOLD)
            y0 = 2
        for i in range(max(0, h - y0 - 2)):
            n = topo + i
            if n >= len(itens):
                break
            realce = curses.A_REVERSE if selecionavel and n == indice else 0
            self._escrever(y0 + i, 0, itens[n].ljust(w - 1) if realce else itens[n], realce)
        self._escrever(h - 2, 0, status, curses.A_BOLD)
        self._escrever(h - 1, 0, rodape, curses.A_DIM)
        self._t.refresh()

    def aviso(self, texto: str) -> None:
        """Mensagem sem esperar tecla (ex.: 'executando...')."""
        h, _ = self.tamanho()
        self._t.erase()
        self._escrever(h // 2, 2, texto, curses.A_BOLD)
        self._t.refresh()

    def modal(self, linhas: list[str], rodape: str = "Pressione qualquer tecla para continuar") -> None:
        self._t.erase()
        for i, l in enumerate(linhas):
            self._escrever(1 + i, 2, l)
        self._escrever(self.tamanho()[0] - 1, 0, rodape, curses.A_DIM)
        self._t.refresh()
        while self.tecla() is None:
            pass

    def pedir_texto(self, prompt: str, linhas_acima: list[str] | None = None) -> str | None:
        """Lê uma linha. Enter confirma, Esc cancela (retorna None)."""
        buf = ""
        while True:
            h, _ = self.tamanho()
            self._t.erase()
            for i, l in enumerate(linhas_acima or []):
                self._escrever(1 + i, 2, l)
            self._escrever(h - 1, 0, prompt + buf + "▏", curses.A_BOLD)
            self._t.refresh()
            t = self.tecla()
            if t is None or t == curses.KEY_RESIZE:
                continue
            if t in ENTER:
                return buf
            if t == ESC:
                return None
            if t in BACKSPACE:
                buf = buf[:-1]
            elif isinstance(t, str) and t.isprintable():
                buf += t


class TuiConfirmacao(ConfirmationPort):
    def __init__(self, terminal: Terminal, plano_linhas: list[str]) -> None:
        self._t = terminal
        self._plano = plano_linhas

    def confirmar(self, pedido: PedidoConfirmacao) -> bool:
        if pedido.tipo is TipoConfirmacao.SIMPLES:
            r = self._t.pedir_texto(f"Risco {pedido.risco.rotulo}. Confirmar? (s/N, Esc cancela): ", self._plano)
            return (r or "").strip().lower() in ("s", "sim")
        r = self._t.pedir_texto(f"Risco {pedido.risco.rotulo}! Digite o IP {pedido.host_ip} para confirmar: ", self._plano)
        return (r or "").strip() == pedido.host_ip


# ------------------------------------------------------------------------- telas
class Tela:
    titulo = ""
    cabecalho: str | None = None
    rodape = "↑↓ navegar  PgUp/PgDn rolar  q/Esc voltar"
    selecionavel = False

    def __init__(self) -> None:
        self.indice = 0
        self.topo = 0

    def itens(self) -> list[str]:
        return []

    def tratar_tecla(self, tecla, app: "NetHubTui") -> bool:
        return False

    def ajustar_topo(self, altura: int) -> None:
        n = len(self.itens())
        self.indice = max(0, min(self.indice, max(0, n - 1)))
        if self.indice < self.topo:
            self.topo = self.indice
        elif self.indice >= self.topo + altura:
            self.topo = self.indice - altura + 1


class TelaTexto(Tela):
    def __init__(self, titulo: str, linhas: list[str]) -> None:
        super().__init__()
        self.titulo = titulo
        self._linhas = linhas

    def itens(self) -> list[str]:
        return self._linhas


class TelaHosts(Tela):
    titulo = "Hosts"
    selecionavel = True
    rodape = "↑↓ navegar  Enter detalhes  d descobrir  h histórico  t filtrar por tag  r atualizar  q sair"

    def __init__(self, app: "NetHubTui") -> None:
        super().__init__()
        self.tag: str | None = None
        self.hosts: list[Host] = []
        self.recarregar(app)

    def recarregar(self, app: "NetHubTui") -> None:
        try:
            self.hosts = app.c.inventario.listar(self.tag)
        except NetHubError as exc:
            self.hosts, app.status = [], f"Erro: {exc}"
        self.titulo = "Hosts" + (f" [tag: {self.tag}]" if self.tag else "")
        linhas = fmt.tabela(fmt.CABECALHO_HOSTS, [fmt.resumo_host(h) for h in self.hosts])
        self.cabecalho = linhas[0]
        self._linhas = linhas[2:]

    def itens(self) -> list[str]:
        return self._linhas or ["(inventário vazio - pressione 'd' para descobrir hosts)"]

    def tratar_tecla(self, tecla, app: "NetHubTui") -> bool:
        if tecla == "d":
            app.descobrir()
            self.recarregar(app)
        elif tecla == "r":
            self.recarregar(app)
            app.status = "Inventário atualizado."
        elif tecla == "t":
            valor = app.t.pedir_texto("Filtrar por tag (vazio = todas): ")
            if valor is not None:
                self.tag = valor.strip().lower() or None
                self.recarregar(app)
        elif tecla == "h":
            app.empilhar(TelaHistorico(app))
        elif tecla in ENTER and self.hosts:
            app.empilhar(TelaDetalheHost(self.hosts[min(self.indice, len(self.hosts) - 1)].ip, app))
        else:
            return False
        return True


class TelaDetalheHost(Tela):
    rodape = "1-9 executar ação  r atualizar  ↑↓ rolar  q/Esc voltar"

    def __init__(self, ip: str, app: "NetHubTui") -> None:
        super().__init__()
        self.ip = ip
        self.titulo = f"Host {ip}"
        self.recarregar(app)

    def recarregar(self, app: "NetHubTui") -> None:
        self.host = app.c.inventario.obter(self.ip)
        self.acoes = app.c.menu.acoes_compativeis(self.host)
        self._linhas = fmt.linhas_host(self.host, self.acoes)

    def itens(self) -> list[str]:
        return self._linhas

    def tratar_tecla(self, tecla, app: "NetHubTui") -> bool:
        if tecla == "r":
            self.recarregar(app)
            return True
        if isinstance(tecla, str) and tecla.isdigit() and tecla != "0":
            n = int(tecla)
            if n > len(self.acoes):
                app.status = "Ação indisponível: capacidade não verificada ou inexistente para este host."
            else:
                app.rodar_acao(self.acoes[n - 1], self.host)
                self.recarregar(app)
            return True
        return False


class TelaHistorico(Tela):
    titulo = "Histórico de operações"
    selecionavel = True
    rodape = "↑↓ navegar  Enter detalhes  a verificar integridade  q/Esc voltar"

    def __init__(self, app: "NetHubTui", alvo: str | None = None) -> None:
        super().__init__()
        self.registros = app.c.historico.consultar(FiltroHistorico(alvo=alvo, limite=200))
        linhas = fmt.tabela(fmt.CABECALHO_HISTORICO, [fmt.linha_historico(e) for e in self.registros])
        self.cabecalho = linhas[0]
        self._linhas = linhas[2:]

    def itens(self) -> list[str]:
        return self._linhas or ["(nenhum registro)"]

    def tratar_tecla(self, tecla, app: "NetHubTui") -> bool:
        if tecla in ENTER and self.registros:
            ev = self.registros[min(self.indice, len(self.registros) - 1)]
            app.empilhar(TelaTexto(f"Registro #{ev.id}", fmt.linhas_evidencia(ev)))
            return True
        if tecla == "a":
            ok, quebrado = app.c.evidencias.verificar_cadeia()
            app.status = "Cadeia de evidências íntegra." if ok else f"ALERTA: inconsistência a partir do registro #{quebrado}!"
            return True
        return False


# --------------------------------------------------------------------- aplicação
class NetHubTui:
    def __init__(self, tela, container: Container) -> None:
        self.t = Terminal(tela)
        self.c = container
        self.status = ""
        self._pilha: list[Tela] = []

    def empilhar(self, tela: Tela) -> None:
        self._pilha.append(tela)

    def voltar(self) -> None:
        if self._pilha:
            self._pilha.pop()

    def executar(self) -> None:
        self.empilhar(TelaHosts(self))
        while self._pilha:
            tela = self._pilha[-1]
            h, _ = self.t.tamanho()
            altura = max(1, h - 3 - (1 if tela.cabecalho else 0))
            tela.ajustar_topo(altura)
            self.t.desenhar(tela.titulo, tela.cabecalho, tela.itens(), tela.indice, tela.topo,
                            tela.selecionavel, tela.rodape, self.status)
            tecla = self.t.tecla()
            if tecla is None:
                continue
            self.status = ""
            try:
                if tela.tratar_tecla(tecla, self):
                    continue
            except NetHubError as exc:
                self.status = f"Erro: {exc}"
                continue
            self._navegar(tela, tecla, altura)

    def _navegar(self, tela: Tela, tecla, altura: int) -> None:
        if tecla in (curses.KEY_UP, "k"):
            tela.indice -= 1
        elif tecla in (curses.KEY_DOWN, "j"):
            tela.indice += 1
        elif tecla == curses.KEY_PPAGE:
            tela.indice -= altura
        elif tecla == curses.KEY_NPAGE:
            tela.indice += altura
        elif tecla in (curses.KEY_HOME, "g"):
            tela.indice = 0
        elif tecla in (curses.KEY_END, "G"):
            tela.indice = len(tela.itens()) - 1
        elif tecla in ("q", ESC):
            self.voltar()

    # -- fluxos
    def descobrir(self) -> None:
        texto = self.t.pedir_texto("Alvo da varredura (IP/CIDR, ex. 192.168.0.0/24; Esc cancela): ")
        if not texto:
            return
        try:
            alvo = AlvoRede.criar(texto)
        except AlvoInvalido as exc:
            self.status = f"Erro: {exc}"
            return
        resp = self.t.pedir_texto(
            f"Você tem autorização para varrer {alvo.cidr} ({alvo.total_enderecos} endereços)? Digite 'sim': "
        )
        if (resp or "").strip().lower() != "sim":
            self.status = "Descoberta cancelada: autorização não confirmada."
            return
        self.t.aviso(f"Executando Nmap em {alvo.cidr}... isso pode levar alguns minutos.")
        try:
            r = self.c.descoberta.descobrir(alvo, autorizado=True)
        except NetHubError as exc:
            self.status = f"Erro: {exc}"
            return
        self.status = f"Descoberta concluída: {len(r.hosts)} host(s) ({r.novos} novo(s), {r.atualizados} atualizado(s))."

    def rodar_acao(self, acao_cls, host: Host) -> None:
        if acao_cls is ExecutarComandoAcao:
            self._executar_comando(host)
        else:  # ponto de extensão: novas ações ganham seu fluxo aqui
            self.status = f"Ação '{acao_cls.nome}' ainda não tem fluxo na TUI."

    def _executar_comando(self, host: Host) -> None:
        comando = self.t.pedir_texto(f"Comando em {host.ip} (não interativo; Esc cancela): ")
        if not comando:
            return
        try:
            acao = ExecutarComandoAcao(comando)
            plano = self.c.executor.simular(acao, host)
        except (ComandoInvalido, NetHubError) as exc:
            self.status = f"Erro: {exc}"
            return
        linhas_plano = fmt.linhas_plano(plano)
        if plano.exige_dry_run:
            self.t.modal(linhas_plano, "Dry-run obrigatório (risco crítico). Pressione uma tecla para prosseguir à confirmação")
        if plano.tipo_confirmacao is TipoConfirmacao.NENHUMA:
            self.t.aviso("Executando...")
        resultado = self.c.executor.executar(
            acao, host, TuiConfirmacao(self.t, linhas_plano), dry_run_realizado=plano.exige_dry_run
        )
        self.empilhar(TelaTexto(f"Resultado - {host.ip}", fmt.linhas_resultado(resultado)))


def executar_tui(container: Container) -> None:
    if not (sys.stdin.isatty() and sys.stdout.isatty()):
        raise NetHubError("A TUI requer um terminal interativo.")
    os.environ.setdefault("ESCDELAY", "25")
    curses.wrapper(lambda tela: NetHubTui(tela, container).executar())
