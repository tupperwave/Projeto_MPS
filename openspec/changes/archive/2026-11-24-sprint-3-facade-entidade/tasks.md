# Tasks

## 1. Especificação e análise

- [x] 1.1 Validar a nova capability e seus cenários para CRUD, associação host/anotação, fachada singleton e contagem.
- [x] 1.2 Criar UC15 e atualizar o diagrama de classes de análise com entidade, gerente e fachada.

## 2. Entidade e persistência

- [x] 2.1 Criar entidade `AnotacaoHost`, erros e porta de repositório com validações e operações de CRUD.
- [x] 2.2 Implementar serviço de aplicação e repositório SQLite com migração, vínculo a host e tratamento de ausências.

## 3. Fachada de gerentes

- [x] 3.1 Adicionar `FacadeSingletonController` ao container com os gerentes do sistema e escopo singleton da composição.
- [x] 3.2 Implementar contagem de hosts, usuários e anotações para cenários vazio e combinado.
- [x] 3.3 Cobrir CRUD, associação, singleton e contagem com testes automatizados.

## 4. Registro da sprint

- [x] 4.1 Atualizar `docs/00-entregas.md` com entregas e pendência de revisão da Sprint 3.
- [x] 4.2 Separar a entrega em três pacotes ZIP ordenados para criação dos commits pelo time.

## 5. Revisão SOLID após a revisão dos specs anteriores

- [x] 5.1 Extrair contagem para serviço de aplicação extensível e manter a porta de contagem segregada das portas CRUD.
- [x] 5.2 Agrupar gerentes na fachada e limitar o Singleton ao escopo de cada composição sem manter instâncias encerradas.
- [x] 5.3 Definir o ciclo de vida das conexões pela composição e fechar containers próprios da CLI.
- [x] 5.4 Atualizar diagrama e testes para refletir as novas dependências e validar os cinco princípios SOLID.
