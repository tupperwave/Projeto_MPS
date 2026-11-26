# Como contribuir

Estas regras registram o fluxo adotado pelo grupo para a disciplina. A branch de integração se chama `dev` (equivalente a `develop`); `main` contém apenas versões entregues e identificadas.

## Branches

- Não faça commits diretamente em `main` nem em `dev`.
- Crie uma branch por tarefa a partir de `dev`, usando `feat/`, `fix/`, `docs/`, `refactor/` ou `test/` seguida de um nome curto em kebab-case. Exemplo: `feat/relatorio-acessos`.
- Integre mudanças em `dev` por Pull Request. O PR deve descrever o problema, resumir a solução e informar como foi verificada; associe-o à tarefa ou issue quando houver.
- Branches `copilot/…` podem ser usadas por agentes, mas suas alterações também devem ser revisadas e integradas por PR.
- Não envie arquivos pelo recurso web “Add files via upload”; faça as alterações em uma branch local e abra um PR.
- Atualize sua branch com `dev` antes de pedir revisão e resolva conflitos nela.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/) no formato `tipo(escopo): descrição`:

- `feat`: funcionalidade; `fix`: correção; `docs`: documentação; `refactor`: mudança interna; `test`: testes; `build` ou `chore`: manutenção.
- Escreva a descrição no imperativo, em minúsculas, sem ponto final. Exemplo: `feat(anotacoes): incluir filtro por host`.
- Mantenha cada commit focado em uma mudança coerente. Não use mensagens vagas como `update`, `Mod1` ou `correções`.

## Verificação e documentação

- Execute os testes pertinentes antes de abrir o PR e informe o resultado na descrição.
- Atualize documentos em `docs/` e requisitos em `openspec/` quando a mudança alterar comportamento ou decisões do projeto.
- Preserve os arquivos-fonte em `docs/03-elicitacao/anexos/fontes/`; registre as mudanças nos documentos organizados e suas origens.
- Use IDs estáveis para RFnn, NF-CAT-nn, UCnn, TLnn e ADR-nnnn. Não reutilize nem renumere IDs publicados.
- Use Mermaid nos diagramas renderizados pelo GitHub.
