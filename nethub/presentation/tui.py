"""Infraestrutura de apresentação da TUI do NetHub.

Este modulo concentra a configuracao visual e as operações de terminal.
Nao deve conhecer servicos de aplicacao, persistencia, descoberta de rede
ou execucao remota.
"""
from __future__ import annotations

import curses
import os
import sys
from typing import Sequence

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
CTRL_L = "\x0c"


class Tema:
    """Centraliza a paleta e a classificação visual de mensagens."""

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
                except curses.error:
                    # Um par indisponível não deve impedir a inicializacao
                    # dos demais nem inviabilizar o modo sem cores.
                    continue
                self._pares_disponiveis.add(numero)

            self.com_cores = bool(self._pares_disponiveis)
        except curses.error:
            self._pares_disponiveis.clear()
            self.com_cores = False

    def atributo(
        self,
        par: int,
        *,
        negrito: bool = False,
        dim: bool = False,
    ) -> int:
        """Retorna atributos visuais, preservando fallback sem cores."""
        atributo = 0

        if self.com_cores and par in self._pares_disponiveis:
            try:
                atributo |= curses.color_pair(par)
            except curses.error:
                # A interface continua funcional se o terminal perder
                # suporte a um par durante a execucao.
                pass

        if negrito:
            atributo |= curses.A_BOLD
        if dim:
            atributo |= curses.A_DIM

        return atributo

    def para_texto(self, texto: str) -> int:
        """Escolhe atributos para indicadores textuais conhecidos."""
        upper = texto.upper()

        if any(
            marcador in upper
            for marcador in ("[ERRO]", "[FALHA]", "[ALERTA]", "CRÍTICO", "CRITICO")
        ):
            return self.atributo(self.ERRO, negrito=True)

        # A verificacao negativa deve preceder DISPONIVEL para evitar
        # classificacao incorreta por substring.
        if any(
            marcador in upper
            for marcador in ("[--]", "INDISPONÍVEL", "INDISPONIVEL")
        ):
            return self.atributo(self.AVISO)

        if any(
            marcador in upper
            for marcador in (
                "[OK]",
                "CONCLUÍDA",
                "CONCLUIDA",
                "DISPONÍVEL",
                "DISPONIVEL",
            )
        ):
            return self.atributo(self.SUCESSO)

        if any(
            marcador in upper
            for marcador in (
                "[?]",
                "[!]",
                "DESCONHECIDA",
                "DESCONHECIDO",
                "MÉDIO",
                "MEDIO",
            )
        ):
            return self.atributo(self.AVISO)

        if any(marcador in upper for marcador in ("BAIXO", "CANCELADA", "CANCELADO")):
            return self.atributo(self.INFO)

        if texto.lstrip().startswith(("#", "ID ", "DATA ", "IP ", "PORTA")):
            return self.atributo(self.DESTAQUE, negrito=True)

        return self.atributo(self.PAINEL)


class Terminal:
    """Encapsula desenho, leitura de teclas e entrada interativa via curses."""

    def __init__(self, tela: curses.window) -> None:
        self._t = tela
        self.tema = Tema()

        try:
            curses.curs_set(0)
        except curses.error:
            # Alguns terminais nao permitem alterar a visibilidade do cursor.
            pass

        try:
            self._t.keypad(True)
        except curses.error:
            # A TUI ainda pode operar com leitura de teclas limitada.
            pass

    def tamanho(self) -> tuple[int, int]:
        """Retorna (altura, largura) da janela atual."""
        try:
            altura, largura = self._t.getmaxyx()
        except curses.error:
            return 0, 0
        return altura, largura

    def tecla(self) -> str | int | None:
        """Lê uma tecla; erros transitorios de curses sao tratados como ausencia."""
        try:
            return self._t.get_wch()
        except curses.error:
            return None

    def solicitar_redesenho(self) -> None:
        """Solicita que curses redesenhe integralmente a janela no proximo refresh."""
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
            # Redimensionamentos podem invalidar uma operacao de desenho.
            pass

    def _escrever(self, y: int, x: int, texto: str, attr: int = 0) -> None:
        """Escreve dentro dos limites visíveis, reservando a ultima coluna."""
        altura, largura = self.tamanho()
        if altura <= 0 or largura <= 1:
            return
        if y < 0 or y >= altura or x < 0 or x >= largura - 1:
            return

        trecho = str(texto)[: max(0, largura - 1 - x)]
        if not trecho:
            return

        try:
            self._t.addstr(y, x, trecho, attr)
        except curses.error:
            # Pode ocorrer com caracteres de largura dupla ou durante resize.
            pass

    def _linha(self, y: int, caractere: str = "-") -> None:
        _, largura = self.tamanho()
        if not caractere:
            return
        self._escrever(
            y,
            0,
            caractere[0] * max(0, largura - 1),
            self.tema.atributo(Tema.SUAVE, dim=True),
        )

    def _cabecalho_global(self, titulo: str) -> None:
        _, largura = self.tamanho()
        texto = f" NetHub  /  {titulo} "

        if largura > 58:
            estado = "? ajuda  "
            area_titulo = max(0, largura - len(estado) - 1)
            texto = texto[:area_titulo].ljust(area_titulo) + estado

        self._escrever(
            0,
            0,
            texto,
            self.tema.atributo(Tema.CABECALHO, negrito=True),
        )

    def desenhar(
        self,
        titulo: str,
        cabecalho: str | None,
        itens: Sequence[str],
        indice: int,
        topo: int,
        selecionavel: bool,
        rodape: str,
        status: str,
    ) -> None:
        """Desenha uma tela generica de lista ou texto."""
        altura, largura = self.tamanho()
        self._limpar()

        if altura < 6 or largura < 24:
            self._escrever(0, 0, "NetHub - terminal pequeno", curses.A_BOLD)
            self._escrever(2, 0, "Aumente a janela para continuar.")
            self._escrever(4, 0, "q/Esc voltar; Ctrl+L redesenhar")
            self._atualizar()
            return

        self._cabecalho_global(titulo)
        self._escrever(
            1,
            1,
            titulo.upper(),
            self.tema.atributo(Tema.DESTAQUE, negrito=True),
        )

        if cabecalho:
            self._escrever(
                2,
                1,
                cabecalho,
                self.tema.atributo(Tema.DESTAQUE, negrito=True),
            )
            self._linha(3)
            primeira_linha = 4
        else:
            self._linha(2)
            primeira_linha = 3

        limite = max(0, altura - primeira_linha - 2)
        for deslocamento in range(limite):
            posicao = topo + deslocamento
            if posicao >= len(itens):
                break

            linha = str(itens[posicao])
            y = primeira_linha + deslocamento
            if selecionavel and posicao == indice:
                self._escrever(
                    y,
                    0,
                    ("> " + linha).ljust(max(0, largura - 1)),
                    self.tema.atributo(Tema.SELECAO, negrito=True),
                )
            else:
                self._escrever(y, 1, linha, self.tema.para_texto(linha))

        if status:
            status_upper = status.upper()
            if any(m in status_upper for m in ("ERRO", "ALERTA", "FALHA")):
                status_attr = self.tema.atributo(Tema.ERRO, negrito=True)
            elif any(
                m in status_upper
                for m in ("CONCLUÍD", "SUCESSO", "ATUALIZADO")
            ):
                status_attr = self.tema.atributo(Tema.SUCESSO, negrito=True)
            else:
                status_attr = self.tema.atributo(Tema.INFO, negrito=True)
            texto_status = " " + status
        else:
            status_attr = self.tema.atributo(Tema.SUAVE, dim=True)
            texto_status = " PRONTO"

        self._escrever(
            altura - 2,
            0,
            texto_status.ljust(max(0, largura - 1)),
            status_attr,
        )
        self._escrever(
            altura - 1,
            0,
            rodape,
            self.tema.atributo(Tema.SUAVE, dim=True),
        )
        self._atualizar()

    def _caixa(self, y: int, x: int, altura: int, largura: int, titulo: str) -> None:
        """Desenha uma caixa retangular respeitando os limites da janela."""
        limite_y, limite_x = self.tamanho()
        if (
            altura < 2
            or largura < 4
            or y < 0
            or x < 0
            or y + altura > limite_y - 1
            or x + largura > limite_x
        ):
            return

        atributo = self.tema.atributo(Tema.INFO, dim=True)
        self._escrever(y, x, "+" + "-" * (largura - 2) + "+", atributo)
        for deslocamento in range(1, altura - 1):
            self._escrever(
                y + deslocamento,
                x,
                "|" + " " * (largura - 2) + "|",
                atributo,
            )
        self._escrever(y + altura - 1, x, "+" + "-" * (largura - 2) + "+", atributo)

        titulo_curto = f" {titulo} "[: max(0, largura - 4)]
        self._escrever(
            y,
            x + 2,
            titulo_curto,
            self.tema.atributo(Tema.DESTAQUE, negrito=True),
        )

    def aviso(self, texto: str) -> None:
        """Exibe um estado transitório antes de uma operacao sincrona."""
        altura, _ = self.tamanho()
        self._limpar()
        self._cabecalho_global("Operação em andamento")
        self._escrever(
            max(1, altura // 2 - 1),
            2,
            texto,
            self.tema.atributo(Tema.AVISO, negrito=True),
        )
        self._escrever(
            altura - 2,
            2,
            "Aguarde a conclusão; não inicie outra operação.",
            self.tema.atributo(Tema.SUAVE, dim=True),
        )
        self._atualizar()

    def modal(
        self,
        linhas: Sequence[str],
        rodape: str = "Pressione qualquer tecla para continuar",
    ) -> None:
        """Exibe um modal e aguarda entrada; resize redesenha o conteúdo."""
        while True:
            altura_tela, largura_tela = self.tamanho()
            self._limpar()

            if altura_tela >= 8 and largura_tela >= 32:
                altura = min(altura_tela - 4, max(5, len(linhas) + 4))
                largura = min(
                    largura_tela - 4,
                    max(24, max((len(linha) for linha in linhas), default=20) + 6),
                )
                y = max(1, (altura_tela - altura) // 2)
                x = max(1, (largura_tela - largura) // 2)
                self._caixa(y, x, altura, largura, "CONFIRMAÇÃO / PLANO")

                for indice, linha in enumerate(linhas[: altura - 3]):
                    self._escrever(
                        y + 1 + indice,
                        x + 2,
                        linha,
                        self.tema.para_texto(linha),
                    )
                self._escrever(
                    y + altura - 2,
                    x + 2,
                    rodape,
                    self.tema.atributo(Tema.SUAVE, dim=True),
                )
            else:
                for indice, linha in enumerate(linhas[: max(0, altura_tela - 3)]):
                    self._escrever(
                        indice + 1,
                        1,
                        linha,
                        self.tema.para_texto(linha),
                    )
                self._escrever(
                    altura_tela - 1,
                    0,
                    rodape,
                    self.tema.atributo(Tema.SUAVE, dim=True),
                )

            self._atualizar()
            tecla = self.tecla()
            if tecla is None or tecla == curses.KEY_RESIZE:
                continue
            return

    def pedir_texto(
        self,
        prompt: str,
        linhas_acima: Sequence[str] | None = None,
        *,
        ocultar: bool = False,
    ) -> str | None:
        """Lê uma linha; Enter confirma, Esc cancela e senhas são mascaradas."""
        buffer = ""

        while True:
            altura, largura = self.tamanho()
            self._limpar()
            self._cabecalho_global("Entrada")

            for indice, linha in enumerate(linhas_acima or ()):
                if indice + 2 >= altura - 3:
                    break
                self._escrever(
                    2 + indice,
                    2,
                    linha,
                    self.tema.para_texto(linha),
                )

            y_prompt = max(1, altura - 3)
            self._linha(y_prompt - 1)
            prompt_visivel = prompt[: max(0, largura - 4)]
            espaco = max(1, largura - len(prompt_visivel) - 4)
            exibido = "*" * len(buffer) if ocultar else buffer
            if len(exibido) > espaco:
                exibido = exibido[-espaco:]

            self._escrever(
                y_prompt,
                1,
                prompt_visivel,
                self.tema.atributo(Tema.DESTAQUE, negrito=True),
            )
            x_valor = min(max(1, largura - 2), len(prompt_visivel) + 1)
            self._escrever(
                y_prompt,
                x_valor,
                exibido + "_",
                self.tema.atributo(Tema.PAINEL, negrito=True),
            )
            self._escrever(
                altura - 1,
                0,
                "Enter confirmar  |  Esc cancelar",
                self.tema.atributo(Tema.SUAVE, dim=True),
            )
            self._atualizar()

            tecla = self.tecla()
            if tecla is None or tecla == curses.KEY_RESIZE:
                continue
            if tecla in ENTER:
                return buffer
            if tecla == ESC:
                return None
            if tecla in BACKSPACE:
                buffer = buffer[:-1]
            elif isinstance(tecla, str) and tecla.isprintable() and tecla not in ("\n", "\r"):
                buffer += tecla
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
    rodape = "↑↓ navegar  Enter detalhes  d descobrir  h histórico  u usuários  t filtrar  r atualizar  q sair"

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
        elif tecla == "u":
            app.empilhar(TelaUsuarios(app))
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


class TelaUsuarios(Tela):
    titulo = "Usuários"
    selecionavel = True
    rodape = "c cadastrar  r atualizar  q/Esc voltar"

    def __init__(self, app: "NetHubTui") -> None:
        super().__init__()
        self.usuarios = []
        self.recarregar(app)

    def recarregar(self, app: "NetHubTui") -> None:
        self.usuarios = app.c.usuarios.listar()
        self._linhas = [usuario.login for usuario in self.usuarios]

    def itens(self) -> list[str]:
        return self._linhas or ["(nenhum usuário cadastrado)"]

    def tratar_tecla(self, tecla, app: "NetHubTui") -> bool:
        if tecla == "r":
            self.recarregar(app)
            app.status = "Lista de usuários atualizada."
        elif tecla == "c":
            login = app.t.pedir_texto("Login (Esc cancela): ")
            if login is None:
                return True
            senha = app.t.pedir_texto("Senha (oculta, Esc cancela): ", ocultar=True)
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
