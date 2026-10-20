# 7. Arquitetura

O diagrama de classes atual sugere separação entre apresentação, aplicação, domínio, portas e adaptadores de infraestrutura. O modelo complementar de gerenciamento de usuários está em [`05-analise.md`](../05-analise.md#modelo-de-classes--gerenciamento-de-usuários) e mostra a seleção entre RAM e SQLite, além dos erros de domínio. Os diagramas lógico e físico e a relação entre requisitos não funcionais e decisões arquiteturais ainda precisam ser elaborados.

Decisões aprovadas devem ser registradas em `adr/` com identificadores `ADR-nnnn`.
