"""TUI do NetHub usando apenas a biblioteca padrão (curses).

A apresentação não acessa SQLite, Nmap ou SSH diretamente: todas as operações
continuam passando pelos serviços injetados pelo Container.
"""
from __future__ import annotations

import curses
import inspect
import os
import sys
from datetime import datetime

from ..bootstrap import Container
from ..domain.actions import ExecutarComandoAcao
from ..domain.enums import (
    StatusCapacidade,
    TipoCapacidade,
    TipoConfirmacao,
)
from ..domain.errors import AlvoInvalido, ComandoInvalido, NetHubError
from ..domain.models import AlvoRede, FiltroHistorico, Host
from ..domain.ports import ConfirmationPort, PedidoConfirmacao
from . import formatacao as fmt

ENTER = ("\n", "\r", curses.KEY_ENTER)
ESC = "\x1b"
BACKSPACE = ("\x7f", "\b", curses.KEY_BACKSPACE)
CTRL_L = "\x0c"


class Tema:
    """Paleta centralizada; a interface continua legível sem suporte a cores."""

    CABECALHO = 1
    PAINEL = 2
    DESTAQUE = 3
    SUCESSO = 4
    AVISO = 5
    ERRO = 6
    SUAVE = 7
    SELECAO = 8
    INFO = 9

    def __init__(self) -> None:
        self._pares_disponiveis: set[int] = set()
        self.com_cores = False
        try:
            if not curses.has_colors():
                return
            curses.start_color()
            try:
                curses.use_default_colors()
                fundo = -1
            except curses.error:
                fundo = curses.COLOR_BLACK
            pares = (
                (self.CABECALHO, curses.COLOR_CYAN, fundo),
                (self.PAINEL, curses.COLOR_WHITE, fundo),
                (self.DESTAQUE, curses.COLOR_CYAN, fundo),
                (self.SUCESSO, curses.COLOR_GREEN, fundo),
                (self.AVISO, curses.COLOR_YELLOW, fundo),
                (self.ERRO, curses.COLOR_RED, fundo),
                (self.SUAVE, curses.COLOR_WHITE, fundo),
                (self.SELECAO, curses.COLOR_BLACK, curses.COLOR_CYAN),
                (self.INFO, curses.COLOR_BLUE, fundo),
            )
            for numero, frente, fundo_par in pares:
                try:
                    curses.init_pair(numero, frente, fundo_par)
                    self._pares_disponiveis.add(numero)
                except curses.error:
                    continue
            self.com_cores = bool(self._pares_disponiveis)
        except curses.error:
            self._pares_disponiveis.clear()
            self.com_cores = False

    def atributo(self, par: int, *, negrito: bool = False, dim: bool = False) -> int:
        attr = 0
        if self.com_cores and par in self._pares_disponiveis:
            try:
                attr = curses.color_pair(par)
            except curses.error:
                pass
        if negrito:
            attr |= curses.A_BOLD
        if dim:
            attr |= curses.A_DIM
        return attr

    def para_texto(self, texto: str) -> int:
        """Colore indicadores sem depender exclusivamente da cor para informar."""
        upper = texto.upper()
        if any(m in upper for m in ("[ERRO]", "[FALHA]", "[ALERTA]", "CRÍTICO", "CRITICO")):
            return self.atributo(self.ERRO, negrito=True)
        # Testar INDISPONÍVEL antes de DISPONÍVEL para não classificar
        # erroneamente o estado negativo como sucesso por substring.
        if any(m in upper for m in ("[--]", "INDISPONÍVEL", "INDISPONIVEL")):
            return self.atributo(self.AVISO)
        if any(m in upper for m in ("[OK]", "CONCLUÍDA", "CONCLUIDA", "DISPONÍVEL", "DISPONIVEL")):
            return self.atributo(self.SUCESSO)
        if any(m in upper for m in ("[?]", "[!]", "DESCONHECIDA", "DESCONHECIDO", "MÉDIO", "MEDIO")):
            return self.atributo(self.AVISO)
        if any(m in upper for m in ("BAIXO", "CANCELADA", "CANCELADO")):
            return self.atributo(self.INFO)
        if texto.lstrip().startswith(("#", "ID ", "DATA ", "IP ", "PORTA")):
            return self.atributo(self.DESTAQUE, negrito=True)
        return self.atributo(self.PAINEL)


class Terminal:
    """Camada fina sobre curses: layout, leitura de teclas e formulários."""

    def __init__(self, tela) -> None:
        self._t = tela
        self.tema = Tema()
        try:
            curses.curs_set(0)
        except curses.error:
            pass
        try:
            self._t.keypad(True)
        except curses.error:
            pass

    def tamanho(self) -> tuple[int, int]:
        try:
            return self._t.getmaxyx()
        except curses.error:
            return 0, 0

    def tecla(self):
        try:
            return self._t.get_wch()
        except curses.error:
            return None

    def solicitar_redesenho(self) -> None:
        try:
            self._t.clearok(True)
        except curses.error:
            pass

    def _limpar(self) -> None:
        try:
            self._t.erase()
        except curses.error:
            pass

    def _atualizar(self) -> None:
        try:
            self._t.refresh()
        except curses.error:
            pass

    def _escrever(self, y: int, x: int, texto: str, attr: int = 0) -> None:
        h, w = self.tamanho()
        if y < 0 or y >= h or x < 0 or x >= w - 1:
            return
        try:
            self._t.addstr(y, x, str(texto)[: max(0, w - 1 - x)], attr)
        except curses.error:
            # curses pode lançar erro quando um caractere ocupa duas colunas
            # ou quando o terminal é redimensionado durante o desenho.
            pass

    def _linha(self, y: int, caractere: str = "-") -> None:
        _, w = self.tamanho()
        self._escrever(y, 0, caractere * max(0, w - 1), self.tema.atributo(Tema.SUAVE, dim=True))

    def _cabecalho_global(self, titulo: str) -> None:
        _, w = self.tamanho()
        texto = f" NetHub  /  {titulo} "
        if w > 58:
            estado = "SQLite conectado  |  ? ajuda "
            texto = texto.ljust(max(0, w - len(estado) - 1)) + estado
        self._escrever(0, 0, texto.ljust(max(0, w - 1)), self.tema.atributo(Tema.CABECALHO, negrito=True))

    def desenhar(
        self, titulo, cabecalho, itens, indice, topo, selecionavel, rodape, status
    ) -> None:
        h, w = self.tamanho()
        self._limpar()
        if h < 6 or w < 24:
            self._escrever(0, 0, "NetHub - terminal pequeno", curses.A_BOLD)
            self._escrever(2, 0, "Aumente a janela para continuar.")
            self._escrever(4, 0, "q/Esc voltar; Ctrl+L redesenhar")
            self._atualizar()
            return

        self._cabecalho_global(titulo)
        self._escrever(1, 1, titulo.upper(), self.tema.atributo(Tema.DESTAQUE, negrito=True))
        if cabecalho:
            self._escrever(2, 1, cabecalho, self.tema.atributo(Tema.DESTAQUE, negrito=True))
            self._linha(3)
            y0 = 4
        else:
            self._linha(2)
            y0 = 3

        limite = max(0, h - y0 - 2)
        for i in range(limite):
            n = topo + i
            if n >= len(itens):
                break
            linha = str(itens[n])
            if selecionavel and n == indice:
                self._escrever(y0 + i, 0, ("> " + linha).ljust(max(0, w - 1)), self.tema.atributo(Tema.SELECAO, negrito=True))
            else:
                attr = self.tema.para_texto(linha)
                self._escrever(y0 + i, 1, linha, attr)

        if status:
            upper = status.upper()
            if "ERRO" in upper or "ALERTA" in upper or "FALHA" in upper:
                status_attr = self.tema.atributo(Tema.ERRO, negrito=True)
            elif "CONCLUÍD" in upper or "SUCESSO" in upper or "ATUALIZADO" in upper:
                status_attr = self.tema.atributo(Tema.SUCESSO, negrito=True)
            else:
                status_attr = self.tema.atributo(Tema.INFO, negrito=True)
            self._escrever(h - 2, 0, (" " + status).ljust(max(0, w - 1)), status_attr)
        else:
            self._escrever(h - 2, 0, " PRONTO".ljust(max(0, w - 1)), self.tema.atributo(Tema.SUAVE, dim=True))
        self._escrever(h - 1, 0, rodape, self.tema.atributo(Tema.SUAVE, dim=True))
        self._atualizar()

    def desenhar_dashboard(
        self,
        titulo: str,
        itens: list[str],
        indice: int,
        metricas: list[tuple[str, str, str]],
        recentes: list[str],
        status: str,
        rodape: str,
    ) -> None:
        h, w = self.tamanho()
        self._limpar()
        if h < 16 or w < 58:
            # Fallback compacto: evita desenhar colunas impraticáveis no terminal pequeno.
            self._cabecalho_global(titulo)
            self._escrever(1, 1, "RESUMO", self.tema.atributo(Tema.DESTAQUE, negrito=True))
            linha = "  |  ".join(f"{rotulo}: {valor}" for rotulo, valor, _ in metricas[:2])
            self._escrever(2, 1, linha)
            self._linha(3)
            for i, item in enumerate(itens):
                y = 4 + i
                if y >= h - 3:
                    break
                attr = self.tema.atributo(Tema.SELECAO, negrito=True) if i == indice else self.tema.para_texto(item)
                self._escrever(y, 1, ("> " if i == indice else "  ") + item, attr)
            self._escrever(h - 2, 0, (" " + status if status else " PRONTO").ljust(max(0, w - 1)), self.tema.atributo(Tema.INFO, negrito=True))
            self._escrever(h - 1, 0, rodape, self.tema.atributo(Tema.SUAVE, dim=True))
            self._atualizar()
            return

        self._cabecalho_global(titulo)
        self._escrever(1, 1, "VISÃO GERAL  /  OPERAÇÃO DE INFRAESTRUTURA", self.tema.atributo(Tema.DESTAQUE, negrito=True))

        margem = 1
        gap = 1
        card_w = max(12, (w - 2 * margem - 3 * gap) // 4)
        card_h = 4
        card_y = 3
        for n, (rotulo, valor, descricao) in enumerate(metricas[:4]):
            x = margem + n * (card_w + gap)
            if x + card_w >= w:
                break
            self._caixa(card_y, x, card_h, card_w, rotulo)
            self._escrever(card_y + 1, x + 2, valor, self.tema.atributo(Tema.DESTAQUE, negrito=True))
            self._escrever(card_y + 2, x + 2, descricao, self.tema.atributo(Tema.SUAVE, dim=True))

        painel_y = card_y + card_h + 2
        painel_h = max(4, h - painel_y - 3)
        menu_w = max(23, min(30, w // 3))
        self._caixa(painel_y, 1, painel_h, menu_w, "MENU PRINCIPAL")
        recent_x = menu_w + 3
        recent_w = w - recent_x - 1
        self._caixa(painel_y, recent_x, painel_h, recent_w, "ATIVIDADE RECENTE")

        for i, item in enumerate(itens):
            y = painel_y + 2 + i
            if y >= painel_y + painel_h - 1:
                break
            if i == indice:
                self._escrever(y, 2, ("> " + item).ljust(max(0, menu_w - 2)), self.tema.atributo(Tema.SELECAO, negrito=True))
            else:
                self._escrever(y, 3, item, self.tema.para_texto(item))

        if recentes:
            max_rec = painel_h - 2
            for i, linha in enumerate(recentes[:max_rec]):
                self._escrever(painel_y + 1 + i, recent_x + 2, linha, self.tema.para_texto(linha))
        else:
            self._escrever(painel_y + 2, recent_x + 2, "Nenhuma evidência registrada ainda.", self.tema.atributo(Tema.SUAVE, dim=True))

        self._escrever(h - 2, 0, (" " + status if status else " PRONTO").ljust(max(0, w - 1)), self.tema.atributo(Tema.INFO, negrito=True))
        self._escrever(h - 1, 0, rodape, self.tema.atributo(Tema.SUAVE, dim=True))
        self._atualizar()

    def _caixa(self, y: int, x: int, altura: int, largura: int, titulo: str) -> None:
        h, w = self.tamanho()
        if altura < 2 or largura < 4 or y < 0 or x < 0 or y + altura > h - 1 or x + largura > w:
            return
        attr = self.tema.atributo(Tema.INFO, dim=True)
        self._escrever(y, x, "+" + "-" * (largura - 2) + "+", attr)
        for linha in range(1, altura - 1):
            self._escrever(y + linha, x, "|" + " " * (largura - 2) + "|", attr)
        self._escrever(y + altura - 1, x, "+" + "-" * (largura - 2) + "+", attr)
        titulo_curto = f" {titulo} "[: max(0, largura - 4)]
        self._escrever(y, x + 2, titulo_curto, self.tema.atributo(Tema.DESTAQUE, negrito=True))

    def aviso(self, texto: str) -> None:
        """Exibe um estado transitório antes de uma operação síncrona."""
        h, _ = self.tamanho()
        self._limpar()
        self._cabecalho_global("Operação em andamento")
        self._escrever(max(1, h // 2 - 1), 2, texto, self.tema.atributo(Tema.AVISO, negrito=True))
        self._escrever(h - 2, 2, "Aguarde a conclusão; não inicie outra operação.", self.tema.atributo(Tema.SUAVE, dim=True))
        self._atualizar()

    def modal(self, linhas: list[str], rodape: str = "Pressione qualquer tecla para continuar") -> None:
        h, w = self.tamanho()
        self._limpar()
        if h >= 8 and w >= 32:
            altura = min(h - 4, max(5, len(linhas) + 4))
            largura = min(w - 4, max(24, max((len(x) for x in linhas), default=20) + 6))
            y = max(1, (h - altura) // 2)
            x = max(1, (w - largura) // 2)
            self._caixa(y, x, altura, largura, "CONFIRMAÇÃO / PLANO")
            for i, linha in enumerate(linhas[: altura - 3]):
                self._escrever(y + 1 + i, x + 2, linha, self.tema.para_texto(linha))
            self._escrever(y + altura - 2, x + 2, rodape, self.tema.atributo(Tema.SUAVE, dim=True))
        else:
            for i, linha in enumerate(linhas[: max(0, h - 3)]):
                self._escrever(i + 1, 1, linha, self.tema.para_texto(linha))
            self._escrever(h - 1, 0, rodape, self.tema.atributo(Tema.SUAVE, dim=True))
        self._atualizar()
        while True:
            tecla = self.tecla()
            if tecla is None or tecla == curses.KEY_RESIZE:
                # Recalcula dimensões e redesenha o conteúdo após resize.
                h, w = self.tamanho()
                self._limpar()
                if h >= 8 and w >= 32:
                    altura = min(h - 4, max(5, len(linhas) + 4))
                    largura = min(w - 4, max(24, max((len(x) for x in linhas), default=20) + 6))
                    y, x = max(1, (h - altura) // 2), max(1, (w - largura) // 2)
                    self._caixa(y, x, altura, largura, "CONFIRMAÇÃO / PLANO")
                    for i, linha in enumerate(linhas[:altura - 3]):
                        self._escrever(y + 1 + i, x + 2, linha, self.tema.para_texto(linha))
                    self._escrever(y + altura - 2, x + 2, rodape,
                                   self.tema.atributo(Tema.SUAVE, dim=True))
                else:
                    for i, linha in enumerate(linhas[:max(0, h - 3)]):
                        self._escrever(i + 1, 1, linha, self.tema.para_texto(linha))
                    self._escrever(h - 1, 0, rodape,
                                   self.tema.atributo(Tema.SUAVE, dim=True))
                self._atualizar()
                continue
            break

    def pedir_texto(
        self, prompt: str, linhas_acima: list[str] | None = None, *, ocultar: bool = False
    ) -> str | None:
        """Lê uma linha; Enter confirma e Esc cancela. Senhas nunca são ecoadas."""
        buf = ""
        while True:
            h, w = self.tamanho()
            self._limpar()
            self._cabecalho_global("Entrada")
            for i, linha in enumerate(linhas_acima or []):
                if i + 2 >= h - 3:
                    break
                self._escrever(2 + i, 2, linha, self.tema.para_texto(linha))
            y_prompt = max(1, h - 3)
            self._linha(y_prompt - 1)
            prompt_visivel = prompt[: max(0, w - 4)]
            espaco = max(1, w - len(prompt_visivel) - 4)
            exibido = "*" * len(buf) if ocultar else buf
            if len(exibido) > espaco:
                exibido = exibido[-espaco:]
            self._escrever(y_prompt, 1, prompt_visivel, self.tema.atributo(Tema.DESTAQUE, negrito=True))
            x_valor = min(w - 2, len(prompt_visivel) + 1)
            self._escrever(y_prompt, x_valor, exibido + "_", self.tema.atributo(Tema.PAINEL, negrito=True))
            self._escrever(h - 1, 0, "Enter confirmar  |  Esc cancelar", self.tema.atributo(Tema.SUAVE, dim=True))
            self._atualizar()
            tecla = self.tecla()
            if tecla is None or tecla == curses.KEY_RESIZE:
                continue
            if tecla in ENTER:
                return buf
            if tecla == ESC:
                return None
            if tecla in BACKSPACE:
                buf = buf[:-1]
            elif isinstance(tecla, str) and tecla.isprintable() and tecla not in ("\n", "\r"):
                buf += tecla


class TuiConfirmacao(ConfirmationPort):
    """Mantém inalterada a política de confirmação definida no domínio."""

    def __init__(self, terminal: Terminal, plano_linhas: list[str]) -> None:
        self._t = terminal
        self._plano = plano_linhas

    def confirmar(self, pedido: PedidoConfirmacao) -> bool:
        if pedido.tipo is TipoConfirmacao.SIMPLES:
            resposta = self._t.pedir_texto(
                f"Risco {pedido.risco.rotulo}. Confirmar? Digite 's' (Esc cancela): ", self._plano
            )
            return (resposta or "").strip().lower() in ("s", "sim")
        resposta = self._t.pedir_texto(
            f"Risco {pedido.risco.rotulo}! Digite o IP {pedido.host_ip} para confirmar: ", self._plano
        )
        return (resposta or "").strip() == pedido.host_ip


# ------------------------------------------------------------------------- telas
class Tela:
    titulo = ""
    cabecalho: str | None = None
    rodape = "↑↓/jk navegar  PgUp/PgDn rolar  Enter selecionar  ? ajuda  q/Esc voltar"
    selecionavel = False

    def __init__(self) -> None:
        self.indice = 0
        self.topo = 0

    def itens(self) -> list[str]:
        return []

    def tratar_tecla(self, _tecla, _app: "NetHubTui") -> bool:
        return False

    def ajustar_topo(self, altura: int) -> None:
        quantidade = len(self.itens())
        self.indice = max(0, min(self.indice, max(0, quantidade - 1)))
        if self.indice < self.topo:
            self.topo = self.indice
        elif self.indice >= self.topo + max(1, altura):
            self.topo = self.indice - max(1, altura) + 1


class TelaTexto(Tela):
    def __init__(self, titulo: str, linhas: list[str]) -> None:
        super().__init__()
        self.titulo = titulo
        self._linhas = linhas

    def itens(self) -> list[str]:
        return self._linhas


class TelaDashboard(Tela):
    titulo = "Dashboard"
    selecionavel = True
    rodape = "↑↓/jk navegar  Enter abrir  d descobrir  h histórico  u usuários  r atualizar  ? ajuda  q sair"
    MENU = ["Hosts do inventário", "Descobrir rede", "Histórico de operações", "Usuários", "Ajuda", "Sair"]

    def __init__(self, app: "NetHubTui") -> None:
        super().__init__()
        self.hosts: list[Host] = []
        self.registros = []
        self.metricas: list[tuple[str, str, str]] = []
        self.recentes: list[str] = []
        self.recarregar(app)

    def itens(self) -> list[str]:
        return self.MENU

    def recarregar(self, app: "NetHubTui") -> None:
        try:
            self.hosts = app.c.inventario.listar()
            inventario_disponivel = True
        except NetHubError as exc:
            self.hosts = []
            inventario_disponivel = False
            app.status = f"Erro ao consultar inventário: {exc}"

        if not inventario_disponivel:
            self.metricas = [
                (rotulo, "—", "indisponível")
                for rotulo in (
                    "HOSTS NO INVENTÁRIO", "SSH DISPONÍVEL",
                    "SSH DESCONHECIDO", "SSH INDISPONÍVEL",
                )
            ]
        else:
            disponiveis = 0
            desconhecidos = 0
            indisponiveis = 0
            for host in self.hosts:
                try:
                    estado = host.status_capacidade(TipoCapacidade.SSH)
                except (AttributeError, ValueError):
                    estado = StatusCapacidade.DESCONHECIDA
                if estado == StatusCapacidade.DISPONIVEL:
                    disponiveis += 1
                elif estado == StatusCapacidade.INDISPONIVEL:
                    indisponiveis += 1
                else:
                    desconhecidos += 1

            self.metricas = [
                ("HOSTS NO INVENTÁRIO", str(len(self.hosts)), "inventariados"),
                ("SSH DISPONÍVEL", str(disponiveis), "capacidade confirmada"),
                ("SSH DESCONHECIDO", str(desconhecidos), "ainda não verificado"),
                ("SSH INDISPONÍVEL", str(indisponiveis), "sem capacidade detectada"),
            ]
        try:
            self.registros = app.c.historico.consultar(FiltroHistorico(limite=5))
            self.recentes = [self._resumo_evidencia(e) for e in self.registros]
        except NetHubError as exc:
            self.registros = []
            self.recentes = [f"[ERRO] Não foi possível consultar histórico: {exc}"]

    @staticmethod
    def _resumo_evidencia(evidencia) -> str:
        timestamp = getattr(evidencia, "timestamp", None)
        if isinstance(timestamp, datetime):
            hora = timestamp.strftime("%H:%M")
        else:
            hora = "--:--"
        status_obj = getattr(evidencia, "status", "desconhecido")
        status = str(getattr(status_obj, "value", status_obj)).upper()
        mapa = {
            "CONCLUIDA": "OK",
            "FALHA": "ERRO",
            "CANCELADA": "CANCELADA",
            "EXPIRADA": "EXPIRADA",
        }
        status = mapa.get(status, status)
        acao_obj = getattr(evidencia, "tipo_acao", "ação")
        acao = str(getattr(acao_obj, "value", acao_obj))
        alvo = str(getattr(evidencia, "alvo", "alvo desconhecido"))
        return f"[{status}] {hora}  {acao}  {alvo}"

    def desenhar(self, app: "NetHubTui") -> None:
        app.t.desenhar_dashboard(
            self.titulo, self.itens(), self.indice, self.metricas, self.recentes,
            app.status, self.rodape,
        )

    def tratar_tecla(self, tecla, app: "NetHubTui") -> bool:
        if tecla == "r":
            self.recarregar(app)
            if not app.status:
                app.status = "Dashboard atualizado."
            return True
        if tecla == "d":
            app.descobrir()
            self.recarregar(app)
            return True
        if tecla == "h":
            app.empilhar(TelaHistorico(app))
            return True
        if tecla == "u":
            app.empilhar(TelaUsuarios(app))
            return True
        if tecla in ENTER:
            escolha = self.indice
            if escolha == 0:
                app.empilhar(TelaHosts(app))
            elif escolha == 1:
                app.descobrir()
                self.recarregar(app)
            elif escolha == 2:
                app.empilhar(TelaHistorico(app))
            elif escolha == 3:
                app.empilhar(TelaUsuarios(app))
            elif escolha == 4:
                app.empilhar(TelaAjuda(self))
            elif escolha == 5:
                app.voltar()
            return True
        return False


class TelaAjuda(Tela):
    titulo = "Ajuda e atalhos"
    rodape = "Pressione q ou Esc para voltar"

    def __init__(self, origem: Tela | None = None) -> None:
        super().__init__()
        self.origem = origem
        nome = origem.__class__.__name__ if origem else ""
        linhas = [
            "NAVEGAÇÃO",
            "  ↑/↓ ou j/k       Mover seleção",
            "  Enter             Abrir/confirmar seleção",
            "  PgUp/PgDn         Rolar uma página",
            "  Home/End ou g/G   Início/fim da lista",
            "  q ou Esc          Voltar à tela anterior",
            "  ?                 Abrir esta ajuda",
            "  Ctrl+L             Redesenhar a tela",
            "",
            "ATALHOS POR CONTEXTO",
            "  Dashboard: d descobrir, h histórico, u usuários, r atualizar",
            "  Hosts:      d descobrir, t filtrar por tag, f filtrar por texto",
            "              h histórico, u usuários, r atualizar, Enter detalhes",
            "  Detalhes:   1-9 executar ação compatível, r atualizar host",
            "  Histórico:  Enter abrir evidência, a verificar integridade",
            "  Usuários:   c cadastrar, r atualizar lista",
            "",
            "SEGURANÇA",
            "  Descoberta exige confirmação explícita de autorização.",
            "  Ações remotas mantêm simulação, análise de risco e confirmações.",
            "  Para risco alto/crítico, confirme digitando o IP do alvo.",
            "  Senhas digitadas na TUI são mascaradas e não são exibidas.",
            "",
            f"Tela de origem: {nome or 'não informada'}",
        ]
        self._linhas = linhas

    def itens(self) -> list[str]:
        return self._linhas


class TelaHosts(Tela):
    titulo = "Hosts"
    selecionavel = True
    rodape = "↑↓ navegar  Enter detalhes  a acesso remoto  d descobrir  t tag  f texto  h histórico  u usuários  r atualizar  ? ajuda  q voltar"

    def __init__(self, app: "NetHubTui") -> None:
        super().__init__()
        self.tag: str | None = None
        self.filtro_texto = ""
        self.hosts: list[Host] = []
        self.recarregar(app)

    def recarregar(self, app: "NetHubTui") -> None:
        try:
            hosts = app.c.inventario.listar(self.tag)
            termo = self.filtro_texto.casefold().strip()
            if termo:
                hosts = [
                    host for host in hosts
                    if termo in host.ip.casefold()
                    or termo in (host.hostname or "").casefold()
                    or any(termo in tag.casefold() for tag in host.tags)
                ]
            self.hosts = hosts
        except NetHubError as exc:
            self.hosts, app.status = [], f"Erro: {exc}"
        filtros = []
        if self.tag:
            filtros.append(f"tag={self.tag}")
        if self.filtro_texto:
            filtros.append(f"texto={self.filtro_texto}")
        self.titulo = "Hosts" + ("  |  " + ", ".join(filtros) if filtros else "")
        linhas = fmt.tabela(fmt.CABECALHO_HOSTS, [fmt.resumo_host(host) for host in self.hosts])
        self.cabecalho = linhas[0] if linhas else "IP / HOSTNAME / SSH / PORTAS / TAGS"
        self._linhas = linhas[2:] if len(linhas) > 2 else []
        self.indice = min(self.indice, max(0, len(self.hosts) - 1))

    def itens(self) -> list[str]:
        if not self.hosts:
            mensagem = "Nenhum host corresponde aos filtros atuais."
            if not self.tag and not self.filtro_texto:
                mensagem = "Inventário vazio. Pressione 'd' para descobrir hosts autorizados."
            return [mensagem]
        return self._linhas

    def tratar_tecla(self, tecla, app: "NetHubTui") -> bool:
        if tecla == "d":
            app.descobrir()
            self.recarregar(app)
        elif tecla == "r":
            self.recarregar(app)
            if not app.status:
                app.status = "Inventário atualizado."
        elif tecla == "t":
            valor = app.t.pedir_texto("Filtrar por tag (vazio remove o filtro): ")
            if valor is not None:
                self.tag = valor.strip().lower() or None
                self.indice = self.topo = 0
                self.recarregar(app)
        elif tecla == "f":
            valor = app.t.pedir_texto("Buscar em IP, hostname e tags (vazio limpa): ")
            if valor is not None:
                self.filtro_texto = valor.strip()
                self.indice = self.topo = 0
                self.recarregar(app)
        elif tecla == "h":
            app.empilhar(TelaHistorico(app))
        elif tecla == "u":
            app.empilhar(TelaUsuarios(app))
        elif tecla == "a" and self.hosts:
            posicao = min(self.indice, len(self.hosts) - 1)
            app.empilhar(TelaAcessoHost(self.hosts[posicao].ip, app))
        elif tecla in ENTER and self.hosts:
            posicao = min(self.indice, len(self.hosts) - 1)
            app.empilhar(TelaDetalheHost(self.hosts[posicao].ip, app))
        else:
            return False
        return True


class TelaDetalheHost(Tela):
    rodape = "1-9 executar ação  a acesso remoto  r atualizar  h histórico do host  ? ajuda  q voltar"

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
            app.status = "Detalhes do host atualizados."
            return True
        if tecla == "h":
            app.empilhar(TelaHistorico(app, alvo=self.ip))
            return True
        if tecla == "a":
            app.empilhar(TelaAcessoHost(self.ip, app))
            return True
        if isinstance(tecla, str) and tecla.isdigit() and tecla != "0":
            numero = int(tecla)
            if numero > len(self.acoes):
                app.status = "Ação indisponível: capacidade não verificada ou inexistente para este host."
            else:
                app.rodar_acao(self.acoes[numero - 1], self.host)
                self.recarregar(app)
            return True
        return False


class TelaUsuarios(Tela):
    titulo = "Usuários locais"
    rodape = "c cadastrar  r atualizar  ? ajuda  q/Esc voltar"

    def __init__(self, app: "NetHubTui") -> None:
        super().__init__()
        self.usuarios = []
        self.recarregar(app)

    def recarregar(self, app: "NetHubTui") -> None:
        self.usuarios = app.c.usuarios.listar()
        self.cabecalho = "LOGIN"
        self._linhas = [f"{usuario.login}" for usuario in self.usuarios]

    def itens(self) -> list[str]:
        return self._linhas or ["nenhum usuário cadastrado."]

    def tratar_tecla(self, tecla, app: "NetHubTui") -> bool:
        if tecla == "r":
            self.recarregar(app)
            app.status = "Lista de usuários atualizada."
        elif tecla == "c":
            login = app.t.pedir_texto("Login (Esc cancela): ")
            if login is None:
                return True
            senha = app.t.pedir_texto("Senha oculta (Esc cancela): ", ocultar=True)
            if senha is None:
                return True
            usuario = app.c.usuarios.cadastrar(login, senha)
            self.recarregar(app)
            app.status = f"Usuário {usuario.login!r} cadastrado com sucesso."
        else:
            return False
        return True


class TelaHistorico(Tela):
    titulo = "Histórico de operações"
    selecionavel = True
    rodape = "↑↓ navegar  Enter detalhes  a verificar integridade  r atualizar  ? ajuda  q voltar"

    def __init__(self, app: "NetHubTui", alvo: str | None = None) -> None:
        super().__init__()
        self.alvo = alvo
        self.recarregar(app)

    def recarregar(self, app: "NetHubTui") -> None:
        self.registros = app.c.historico.consultar(FiltroHistorico(alvo=self.alvo, limite=200))
        linhas = fmt.tabela(fmt.CABECALHO_HISTORICO, [fmt.linha_historico(ev) for ev in self.registros])
        self.cabecalho = linhas[0] if linhas else "ID / DATA / ALVO / AÇÃO / RISCO / RESULTADO"
        self._linhas = linhas[2:] if len(linhas) > 2 else []
        if self.alvo:
            self.titulo = f"Histórico · {self.alvo}"

    def itens(self) -> list[str]:
        return self._linhas or ["Nenhum registro de operação encontrado."]

    def tratar_tecla(self, tecla, app: "NetHubTui") -> bool:
        if tecla in ENTER and self.registros:
            ev = self.registros[min(self.indice, len(self.registros) - 1)]
            app.empilhar(TelaTexto(f"Evidência #{ev.id}", fmt.linhas_evidencia(ev)))
            return True
        if tecla == "r":
            self.recarregar(app)
            app.status = "Histórico atualizado."
            return True
        if tecla == "a":
            ok, quebrado = app.c.evidencias.verificar_cadeia()
            app.status = (
                "[OK] Cadeia de evidências íntegra."
                if ok
                else f"[ALERTA] Inconsistência a partir do registro #{quebrado}!"
            )
            return True
        return False


class TelaAcessoHost(Tela):
    """Tela de entrada para as operações remotas disponíveis no host."""

    titulo = "Acesso remoto"
    cabecalho = ""
    selecionavel = True
    rodape = "↑↓ navegar  Enter executar  c usuário  k chave  s senha  u atualizar  x remover  t testar  ? ajuda  q voltar"

    def __init__(self, ip: str, app: "NetHubTui") -> None:
        super().__init__()
        self.ip = ip
        self.host: Host | None = None
        self.acoes = []
        self.recarregar(app)

    def recarregar(self, app: "NetHubTui") -> None:
        self.host = app.c.inventario.obter(self.ip)
        self.acoes = app.c.menu.acoes_compativeis(self.host)
        resumo = fmt.resumo_host(self.host)
        self.cabecalho = f"{self.ip}  |  {resumo}"
        self._linhas = [
            f"{numero}. {acao.nome}"
            for numero, acao in enumerate(self.acoes, start=1)
        ]
        if not self._linhas:
            self._linhas = [
                "Nenhuma ação remota compatível foi encontrada.",
                "Verifique a capacidade SSH do host e atualize a tela.",
            ]
        self.indice = min(self.indice, max(0, len(self.acoes) - 1))

    def itens(self) -> list[str]:
        return self._linhas

    def tratar_tecla(self, tecla, app: "NetHubTui") -> bool:
        if tecla == "r":
            self.recarregar(app)
            app.status = "Acesso remoto atualizado."
            return True
        if tecla in ENTER and self.acoes and self.host is not None:
            acao = self.acoes[min(self.indice, len(self.acoes) - 1)]
            app.rodar_acao(acao, self.host)
            self.recarregar(app)
            return True
        if tecla in ("c", "k", "s", "u", "x", "t"):
            app.gerenciar_acesso(self.ip, tecla)
            self.recarregar(app)
            return True
        return False


class NetHubTui:
    """Orquestra a navegação e mantém a apresentação desacoplada do backend."""

    def __init__(self, tela, container: Container) -> None:
        self.t = Terminal(tela)
        self.c = container
        self.status = ""
        self._pilha: list[Tela] = []

    def _servico_acesso(self):
        """Obtém o serviço de acessos sem acoplar a TUI ao nome do atributo."""
        for nome in ("acessos", "acesso_ssh", "credenciais"):
            servico = getattr(self.c, nome, None)
            if servico is not None:
                return servico
        raise NetHubError("Serviço de acessos SSH não está configurado.")

    def _chamar_acesso(self, operacao: str, **dados):
        servico = self._servico_acesso()
        nomes = {
            "c": ("cadastrar_usuario_ssh", "cadastrar_usuario"),
            "k": ("cadastrar_chave_privada", "cadastrar_chave"),
            "s": ("cadastrar_senha",),
            "u": ("atualizar_acesso", "atualizar"),
            "x": ("remover_acesso", "remover"),
            "t": ("testar_conexao", "testar"),
        }
        metodo = next((getattr(servico, nome, None) for nome in nomes[operacao]), None)
        if metodo is None:
            raise NetHubError(f"Operação de acesso SSH '{operacao}' não está disponível.")

        # Aceita serviços com assinaturas diferentes sem enviar dados extras.
        try:
            parametros = inspect.signature(metodo).parameters
            aceitos = {
                nome: valor for nome, valor in dados.items()
                if nome in parametros or any(
                    p.kind is inspect.Parameter.VAR_KEYWORD for p in parametros.values()
                )
            }
            return metodo(**aceitos)
        except (TypeError, ValueError):
            return metodo(**dados)

    def gerenciar_acesso(self, ip: str, operacao: str) -> None:
        """Executa cadastro, manutenção e teste de uma credencial SSH."""
        login = self.t.pedir_texto(f"Usuário SSH para {ip}: ")
        if login is None or not login.strip():
            return
        dados = {"host_ip": ip, "ip": ip, "login": login.strip(), "usuario": login.strip()}
        if operacao == "c":
            dados["senha"] = self.t.pedir_texto("Senha do usuário SSH (oculta): ", ocultar=True)
            if dados["senha"] is None:
                return
        elif operacao == "k":
            dados["chave_privada"] = self.t.pedir_texto("Caminho ou conteúdo da chave privada: ")
            if not dados["chave_privada"]:
                return
        elif operacao == "s":
            dados["senha"] = self.t.pedir_texto("Senha SSH (oculta): ", ocultar=True)
            if dados["senha"] is None:
                return
        elif operacao == "x":
            confirma = self.t.pedir_texto("Digite REMOVER para confirmar: ")
            if (confirma or "").strip().upper() != "REMOVER":
                self.status = "Remoção de acesso cancelada."
                return
        try:
            resultado = self._chamar_acesso(operacao, **dados)
        except NetHubError:
            raise
        except (OSError, ValueError) as exc:
            self.status = f"Falha no acesso SSH: {exc}"
            return
        if operacao == "t":
            self.status = "[OK] Conexão SSH testada com sucesso."
        else:
            nomes = {"c": "Usuário SSH", "k": "Chave privada", "s": "Senha", "u": "Acesso", "x": "Acesso"}
            self.status = f"{nomes[operacao]} atualizado com sucesso." if operacao == "u" else f"{nomes[operacao]} processado com sucesso."

    def empilhar(self, tela: Tela) -> None:
        self._pilha.append(tela)

    def voltar(self) -> None:
        if self._pilha:
            self._pilha.pop()

    def executar(self) -> None:
        self.empilhar(TelaDashboard(self))
        while self._pilha:
            tela = self._pilha[-1]
            h, _ = self.t.tamanho()
            tem_cabecalho = bool(getattr(tela, "cabecalho", None))
            altura = max(1, h - (4 if tem_cabecalho else 3) - 2)
            tela.ajustar_topo(altura)
            if isinstance(tela, TelaDashboard):
                tela.desenhar(self)
            else:
                self.t.desenhar(
                    tela.titulo, tela.cabecalho, tela.itens(), tela.indice, tela.topo,
                    tela.selecionavel, tela.rodape, self.status,
                )
            tecla = self.t.tecla()
            if tecla is None:
                continue
            if tecla == curses.KEY_RESIZE:
                continue
            if tecla == CTRL_L:
                self.t.solicitar_redesenho()
                continue
            self.status = ""
            try:
                if tecla == "?" and not isinstance(tela, TelaAjuda):
                    self.empilhar(TelaAjuda(tela))
                    continue
                if tela.tratar_tecla(tecla, self):
                    continue
            except NetHubError as exc:
                self.status = f"Erro: {exc}"
                continue
            except (ValueError, OSError) as exc:
                self.status = f"Falha na operação: {exc}"
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

    # -- fluxos já suportados pelo backend
    def descobrir(self) -> None:
        texto = self.t.pedir_texto(
            "Alvo da varredura (IP/CIDR, ex. 192.168.0.0/24): "
        )
        if not texto:
            return
        try:
            alvo = AlvoRede.criar(texto)
        except AlvoInvalido as exc:
            self.status = f"Erro: {exc}"
            return
        resposta = self.t.pedir_texto(
            f"Autorizado a varrer {alvo.cidr} ({alvo.total_enderecos} endereços)? Digite 'sim': "
        )
        if (resposta or "").strip().lower() != "sim":
            self.status = "Descoberta cancelada: autorização não confirmada."
            return

        self.t.aviso(f"Executando Nmap em {alvo.cidr}... pode levar alguns minutos.")
        try:
            resultado = self.c.descoberta.descobrir(alvo, autorizado=True)
        except NetHubError as exc:
            self.status = f"Erro na descoberta: {exc}"
            return
        self.status = (
            f"Descoberta concluída: {len(resultado.hosts)} host(s), "
            f"{resultado.novos} novo(s), {resultado.atualizados} atualizado(s)."
        )

    def rodar_acao(self, acao_cls, host: Host) -> None:
        if acao_cls is ExecutarComandoAcao:
            self._executar_comando(host)
        else:
            # Não anunciar ações que ainda não possuem fluxo implementado nesta TUI.
            self.status = f"A ação '{acao_cls.nome}' ainda não possui fluxo na TUI."

    def _executar_comando(self, host: Host) -> None:
        comando = self.t.pedir_texto(f"Comando em {host.ip} (não interativo): ")
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
            self.t.modal(
                linhas_plano,
                "Dry-run obrigatório. Pressione uma tecla para continuar à confirmação.",
            )
        if plano.tipo_confirmacao is TipoConfirmacao.NENHUMA:
            self.t.aviso(f"Executando comando via SSH em {host.ip}...")
        resultado = self.c.executor.executar(
            acao,
            host,
            TuiConfirmacao(self.t, linhas_plano),
            dry_run_realizado=plano.exige_dry_run,
        )
        self.empilhar(TelaTexto(f"Resultado da execução · {host.ip}", fmt.linhas_resultado(resultado)))


def executar_tui(container: Container) -> None:
    if not (sys.stdin.isatty() and sys.stdout.isatty()):
        raise NetHubError("A TUI requer um terminal interativo.")
    os.environ.setdefault("ESCDELAY", "25")
    curses.wrapper(lambda tela: NetHubTui(tela, container).executar())
