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

As anotações ficam associadas a um host já cadastrado no inventário e são persistidas no SQLite. Primeiro, descubra o host em uma rede para a qual você tenha autorização:

```bash
python3 -m nethub descobrir 192.168.0.0/24 --autorizado
```

O CRUD de anotações está disponível pelo serviço de aplicação, acessado pela fachada `FacadeSingletonController`. A CLI ainda não oferece subcomandos para criar, consultar, atualizar ou excluir anotações. Exemplo de uso pela API Python:

```python
from nethub.bootstrap import criar_container

container = criar_container()
try:
    anotacoes = container.facade.gerentes.anotacoes

    criada = anotacoes.criar("192.168.0.10", "Servidor de arquivos")
    print(criada.id, criada.conteudo)

    print(anotacoes.listar(host_ip="192.168.0.10"))
    anotacoes.atualizar(criada.id, "Servidor de arquivos — rack 2")
    anotacoes.excluir(criada.id)

    print(container.facade.quantidade_entidades_cadastradas())
finally:
    container.fechar()
```

Substitua o IP por um host presente no inventário. A contagem da fachada soma hosts, usuários e anotações cadastrados.
