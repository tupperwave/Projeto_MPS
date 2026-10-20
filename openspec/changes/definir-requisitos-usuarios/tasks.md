# Tasks

## 1. Modelo e validação

- [x] 1.1 Criar `Usuario` com validação de login e senha conforme RF06 e erros de domínio específicos; revisar se cada regra do RF06 termina na exceção específica correspondente.
- [x] 1.2 Armazenar apenas hash com salt aleatório e oferecer verificação de senha em tempo constante; revisar se a senha original não é mantida nem exibida pela representação da entidade.

## 2. Cadastro e listagem em memória

- [x] 2.1 Definir a porta de repositório de usuários e implementar adaptador em memória com coleção; revisar os caminhos de gravação, consulta, listagem e rejeição de login duplicado.
- [x] 2.2 Implementar serviço de cadastro/listagem e injetá-lo no container usando o adaptador em memória; revisar o fluxo pela camada de aplicação e confirmar que a representação pública não exibe hashes.

## 3. Persistência permanente

- [x] 3.1 Implementar repositório SQLite para usuários, com migração e conversão de falhas de persistência; verificar gravação, leitura e propagação segura de erros.

## 4. Seleção do mecanismo

- [x] 4.1 Permitir selecionar RAM ou SQLite durante a inicialização; verificar que o container usa o repositório selecionado.

## 5. Interface e documentação de análise

- [x] 5.1 Integrar cadastro e listagem à CLI/TUI sem mostrar senha ou hash; verificar mensagens de sucesso, lista vazia e erro.
- [x] 5.2 Atualizar diagramas de casos de uso e classes com fronteira, controle, entidade, exceções e os dois mecanismos; conferir rastreabilidade com RF06, RF16 e RF17.

## 6. Cobertura de testes

- [x] 6.1 Cobrir validações, hash, cadastro/listagem em RAM, duplicidade, SQLite e falhas de persistência; verificar a suíte completa com `python3 -m unittest discover -v`.

## 7. Conferência da rubrica das Sprints 1 e 2

- [x] 7.1 Mostrar pelo menos dois perfis de ator no diagrama de casos de uso e marcar fronteira, controle e entidade no diagrama de classes de análise.
- [x] 7.2 Especificar proteção das credenciais como requisito não funcional com cenários verificáveis e cobertura correspondente nos testes.
- [x] 7.3 Registrar que não houve elicitação formal, sem atribuir técnicas, datas ou participantes inexistentes.

## 8. Correções após verificação da rubrica

- [x] 8.1 Contar qualquer caractere não alfanumérico no grupo especial da política de senha e cobrir a regra com teste de regressão.
- [x] 8.2 Converter falhas ao criar diretórios, abrir a conexão e executar migrações SQLite em erro tratável pela CLI; cobrir falha de inicialização com teste.
