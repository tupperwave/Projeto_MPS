# Tasks

## 1. Modelo e validação

- [x] 1.1 Criar `Usuario` com validação de login e senha conforme RF06 e erros de domínio específicos; revisar se cada regra do RF06 termina na exceção específica correspondente.
- [x] 1.2 Armazenar apenas hash com salt aleatório e oferecer verificação de senha em tempo constante; revisar se a senha original não é mantida nem exibida pela representação da entidade.

## 2. Cadastro e listagem em memória

- [ ] 2.1 Definir a porta de repositório de usuários e implementar adaptador em memória com coleção; verificar gravação, consulta, listagem e rejeição de login duplicado.
- [ ] 2.2 Implementar serviço de cadastro/listagem e injetá-lo no container usando o adaptador em memória; verificar que cadastrar e listar pela camada de aplicação produz os usuários esperados sem expor hashes na saída pública.

## 3. Persistência permanente

- [ ] 3.1 Implementar repositório SQLite para usuários, com migração e conversão de falhas de persistência; verificar gravação, leitura e propagação segura de erros.

## 4. Seleção do mecanismo

- [ ] 4.1 Permitir selecionar RAM ou SQLite durante a inicialização; verificar que o container usa o repositório selecionado.

## 5. Interface e documentação de análise

- [ ] 5.1 Integrar cadastro e listagem à CLI/TUI sem mostrar senha ou hash; verificar mensagens de sucesso, lista vazia e erro.
- [ ] 5.2 Atualizar diagramas de casos de uso e classes com fronteira, controle, entidade, exceções e os dois mecanismos; conferir rastreabilidade com RF06, RF16 e RF17.

## 6. Cobertura de testes

- [ ] 6.1 Cobrir validações, hash, cadastro/listagem em RAM, duplicidade, SQLite e falhas de persistência; verificar a suíte completa com `python3 -m unittest discover -v`.
