# 4. Casos de uso

## Casos detalhados disponíveis

- [UC01](uc01.md)
- [UC02](uc02.md)
- [UC03](uc03.md)
- [UC04](uc04.md)
- [UC05](uc05.md)
- [UC06](uc06.md)
- [UC07](uc07.md)
- [UC08](uc08.md)
- [UC09](uc09.md)
- [UC10](uc10.md) — Cadastrar usuário
- [UC11](uc11.md) — Listar usuários
- [UC12](uc12.md)
- [UC13](uc13.md)
- [UC14](uc14.md)

## Diagrama de casos de uso — gerenciamento de usuários

```mermaid
flowchart LR
    administrador([Administrador])
    operador([Operador autorizado])
    subgraph nethub[NetHub]
        iniciar[Selecionar persistência na inicialização - RF17]
        cadastrar[Cadastrar usuário - UC10 / RF06]
        listar[Listar usuários - UC11 / RF16]
        validacao[Validar login e senha]
        erros[Informar validação, duplicidade ou falha de persistência]
        repo[(Repositório selecionado)]
        memoria[Memória RAM]
        sqlite[(SQLite)]
    end
    administrador --> iniciar
    administrador --> cadastrar
    administrador --> listar
    operador --> cadastrar
    operador --> listar
    cadastrar --> validacao --> repo
    listar --> repo
    repo --> memoria
    repo --> sqlite
    validacao -. entrada inválida .-> erros
    repo -. falha ou login duplicado .-> erros
    repo -->|retorna somente login| listar
```

O diagrama cobre RF06, RF16 e RF17. O diagrama geral do produto ainda precisa ser elaborado e revisado pela equipe.

Os dois atores são perfis de uso previstos nos casos de uso; autenticação e autorização por perfil não fazem parte desta implementação.
