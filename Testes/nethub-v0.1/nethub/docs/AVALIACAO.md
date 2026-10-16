# Como avaliar o diferencial do NetHub

**Hipótese:** modelar a infraestrutura como *capacidades verificadas, com risco explícito e evidência de execução*
reduz erros e etapas em tarefas operacionais de pequenos ambientes, comparado ao uso direto da CLI.

## 1. Duas camadas de avaliação

### A) Verificação técnica do diferencial (já automatizada)
`bash tests/e2e.sh` demonstra, de forma reproduzível, o que a CLI "pura" **não** faz:

| Propriedade do diferencial | Demonstração no teste | CLI pura (ssh/systemctl) |
|---|---|---|
| Capacidade provada vs. suposta | `inspect` distingue AUTORIZADA / AUTENTICADA / ALCANÇÁVEL / sem SSH | Porta 22 aberta ≠ acesso; só descobre ao tentar |
| Bloqueio antecipado com motivo | `restart` sem sudo → exit 3 com explicação, sem tentar | Falha na execução |
| Risco proporcional | MÉDIO `--yes`; ALTO exige digitar o alvo; CRÍTICO exige dry-run do **mesmo** plano | Nenhum |
| Validação pós-ação | Serviço `frozen` (restart retorna 0 mas nada muda) é reprovado | `systemctl restart` retorna 0 e acabou |
| Integridade de transferência | SHA-256 origem × destino em todo upload/download | `scp/sftp` não verificam |
| Evidência verificável | Cadeia de hashes; adulteração do banco → `audit verify` falha | Histórico do shell, editável |

### B) Experimento com participantes (a fazer)
- **Desenho:** within-subjects, **contrabalanceado** (metade começa pela CLI, metade pelo NetHub).
- **Amostra:** 5–10 pessoas; declare como estudo **exploratório**. Defina o critério de "sem experiência avançada" *antes* (ex.: nunca administrou servidor Linux em produção).
- **Tarefas (mesmo laboratório, scripts idênticos para os dois braços):**
  1. Descobrir os servidores da rede e dizer em quais é possível administrar (autenticar + sudo).
  2. Verificar a saúde de um servidor e dizer se está operacional.
  3. Enviar um arquivo a um servidor e **comprovar** que chegou íntegro.
  4. Reiniciar um serviço e **comprovar** que reiniciou (tarefa-armadilha: `frozen`).
  5. Executar a verificação pré-atualização e decidir GO/NO-GO.
- **Métricas:** tempo por tarefa, nº de comandos/ações, erros (comando errado, conclusão falsa), taxa de conclusão correta, **SUS**.
- **Medida-chave do diferencial:** *conclusões falsas* (declarar sucesso sem comprovar) na tarefa 4 e 3 — é aí que validação e evidência devem fazer diferença.
- **Registro:** a trilha do NetHub já mede etapas e tempo (`nethub history`); para a CLI use `script`/`history` com timestamp.
- **Ameaças à validade:** amostra pequena; familiaridade do autor com a ferramenta; lab artificial; `systemctl` emulado (use VMs reais no experimento).

## 2. Laboratório recomendado para o experimento
Troque o loopback por 3 VMs Ubuntu/Debian reais (1 com sudo sem senha, 1 sem sudo, 1 com chave não autorizada) + 1 serviço
que reinicia "com sucesso" sem mudar (o `frozen` do lab é equivalente a um serviço `Type=oneshot` mal configurado).
