# NetHub — administração de infraestrutura pelo terminal

Repositório da equipe para a disciplina Métodos e Projeto de Software. A documentação acompanha o desenvolvimento do NetHub, uma proposta de ferramenta TUI para descoberta, inventário e operação de infraestrutura.

## Equipe

- André Luiz Rodrigues Castro da Nóbrega
- Marcos José Edu Ekua Ayingono
- Lucas Martins de Lima Oliveira

## Navegação

- [Entregas da disciplina](docs/00-entregas.md)
- [Visão do produto](docs/01-visao.md)
- [Glossário](docs/02-glossario.md)
- [Elicitação](docs/03-elicitacao.md) e [anexos/fontes](docs/03-elicitacao/anexos/)
- [Casos de uso](docs/04-casos-de-uso/)
- [Análise](docs/05-analise.md)
- [Interface](docs/06-interface/)
- [Arquitetura e ADRs](docs/07-arquitetura/)
- [Projeto](docs/08-projeto.md)
- [Requisitos vigentes](openspec/specs/)

Os arquivos-fonte originais e o PDF anterior foram preservados em `docs/03-elicitacao/anexos/`. Onde as fontes divergem ou não definem critérios suficientes, a documentação sinaliza a pendência em vez de inventar uma decisão.

## Gerenciar usuários

O cadastro solicita a senha sem exibi-la; a listagem mostra somente os logins.

Na raiz do repositório, execute diretamente pelo Python:

```bash
python3 -m nethub --persistencia-usuarios sqlite usuarios cadastrar ana
python3 -m nethub --persistencia-usuarios sqlite usuarios listar
```

Para habilitar o comando `nethub` no shell, instale o projeto na raiz do repositório com `python3 -m pip install -e .`. Depois, os comandos podem ser chamados sem `python3 -m nethub`.

Use `--persistencia-usuarios memoria` para manter os cadastros apenas durante a execução. Sem a opção, a CLI usa memória.

## Gerenciar anotações de hosts — Sprint 3

As anotações ficam associadas a um host já cadastrado no inventário e são persistidas no SQLite. Primeiro, descubra o host em uma rede para a qual você tenha autorização. A descoberta usa o Nmap; no Fedora, instale-o com `sudo dnf install nmap`.

```bash
python3 -m nethub descobrir --rede-local
```

Use os comandos abaixo para criar, listar, consultar, atualizar e excluir anotações. O IP deve corresponder a um host que já esteja no inventário; coloque o conteúdo entre aspas se tiver espaços:

```bash
python3 -m nethub anotacoes criar 192.168.0.10 "Servidor de arquivos"
python3 -m nethub anotacoes listar --host 192.168.0.10
python3 -m nethub anotacoes obter 1
python3 -m nethub anotacoes atualizar 1 "Servidor de arquivos — rack 2"
python3 -m nethub anotacoes excluir 1
```

Os identificadores são retornados ao criar e ao listar anotações. O CRUD é encaminhado pela `FacadeSingletonController`, cuja contagem soma hosts, usuários e anotações cadastrados. Por padrão, as anotações são salvas no banco SQLite do usuário; para selecionar outro arquivo, passe `--db CAMINHO` antes do subcomando, por exemplo: `python3 -m nethub --db ./nethub.db anotacoes listar`.

## Descobrir hosts

A descoberta usa o executável Nmap. No Fedora, instale-o uma vez com `sudo dnf install nmap`; a detecção automática da rede também requer o utilitário `ip` (pacote `iproute`, normalmente já instalado no Fedora). O NetHub verifica se o Nmap está disponível e informa o comando de instalação caso esteja ausente; ele não instala programas do sistema automaticamente.

Para detectar a rede associada à rota IPv4 padrão, mostrar o CIDR e confirmar antes de varrer:

```bash
python3 -m nethub descobrir --rede-local
```

Também é possível antecipar a confirmação, mantendo a descoberta restrita à rede detectada:

```bash
python3 -m nethub descobrir --rede-local --autorizado
```

Se preferir, informe o alvo manualmente como antes: `python3 -m nethub descobrir 192.168.1.0/24`. A rede detectada pode ser uma VPN ou túnel; confira o CIDR exibido antes de autorizar. Se nenhum host responder, o comando orienta a verificar se o segmento está correto.
