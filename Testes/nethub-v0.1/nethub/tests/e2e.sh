#!/bin/bash
# Testes ponta a ponta do NetHub contra o laboratório local (lab/setup_lab.sh).
# Cobre os cenários do TCC: A) descoberta  B) capacidades/administração  C) arquivos  D) workflows,
# além de risco/confirmação e integridade da auditoria.
# Uso (como root, em VM/contêiner descartável):  bash tests/e2e.sh
cd "$(dirname "$0")/.." || exit 1
[ "${SKIP_LAB:-0}" = 1 ] || bash lab/setup_lab.sh </dev/null >/dev/null 2>&1 || { echo "falha ao subir o laboratório"; exit 1; }
BIN="${BIN:-./target/release/nethub}"; [ -x "$BIN" ] || BIN=./target/debug/nethub
export NETHUB_DB="$(mktemp -u /tmp/nh_e2e_XXXXXX.db)" NETHUB_SSH_OPTS="-oUserKnownHostsFile=/tmp/nhlab/known_hosts"
K=/tmp/nhlab/operator_key; T=/tmp/nh_e2e_files; rm -rf "$T"; mkdir -p "$T"
pass=0; fail=0
N() { timeout 120 "$BIN" --no-color "$@" </dev/null 2>&1; }
# check "descrição" CODIGO_ESPERADO "regex" comando...
check() {
  local desc="$1" want="$2" re="$3"; shift 3
  local out code; out="$("$@" 2>&1 </dev/null)"; code=$?
  if [ "$code" = "$want" ] && echo "$out" | grep -Eq -- "$re"; then pass=$((pass+1)); printf '  ok    %s\n' "$desc"
  else fail=$((fail+1)); printf '  FALHA %s (saída %s, esperado %s, padrão /%s/)\n' "$desc" "$code" "$want" "$re"; echo "$out" | head -8 | sed 's/^/        | /'; fi
}
nh() { timeout 120 "$BIN" --no-color "$@"; }

echo "A) Descoberta"
check "scan grava 4 hosts no inventário"          0 "4 host"            nh scan 127.0.0.1-4 --ports 2222,8080 --yes
check "alvo inválido é rejeitado"                 2 "inválid"           nh scan --yes -- -sS
check "alvo público exige digitar o alvo (ALTO)"  4 "ALTO"              nh scan 8.8.8.8 --yes
nh host set 127.0.0.1 --user nhadmin --key $K >/dev/null </dev/null
nh host set 127.0.0.2 --user nhuser  --key $K >/dev/null </dev/null
nh host set 127.0.0.3 --user nokey   --key $K >/dev/null </dev/null

echo "B) Capacidades verificadas e administração"
check "sudo sem senha → AUTORIZADA"               0 "AUTORIZADA"        nh inspect 127.0.0.1
check "sem sudo → AUTENTICADA"                    0 "AUTENTICADA"       nh inspect 127.0.0.2
check "sem chave → apenas ALCANÇÁVEL"             1 "ALCANÇÁVEL"        nh inspect 127.0.0.3
check "sem SSH → falha de conexão registrada"     1 "falha de conexão"  nh inspect 127.0.0.4
check "menu mostra ações bloqueadas com motivo"   0 "Bloqueadas"        nh actions 127.0.0.2
check "restart sem sudo é bloqueado (exit 3)"     3 "bloqueado"         nh service restart 127.0.0.2 nginx --yes
check "saúde determinística"                      0 "Resumo"            nh health 127.0.0.1
check "comando somente leitura (BAIXO)"           0 "exit code: 0"      nh exec 127.0.0.1 -- 'df -h / | tail -1'
check "MÉDIO sem confirmação → exit 4"            4 "MÉDIO"             nh exec 127.0.0.1 -- touch /home/nhadmin/e2e_marker
check "MÉDIO com --yes executa"                   0 "exit code: 0"      nh exec 127.0.0.1 --yes -- touch /home/nhadmin/e2e_marker
check "ALTO: --yes não basta"                     4 "ALTO"              nh exec 127.0.0.1 --yes -- sudo apt-get install -y nginx
check "CRÍTICO: --confirm sem dry-run → exit 4"   4 "dry-run"           nh exec 127.0.0.1 --confirm 127.0.0.1 -- rm -rf /home/nhadmin/e2e_dir
check "CRÍTICO: dry-run registra o plano"         0 "plano [0-9a-f]+"   nh exec 127.0.0.1 --dry-run -- rm -rf /home/nhadmin/e2e_dir
check "CRÍTICO: comando diferente não herda o dry-run" 4 "dry-run"      nh exec 127.0.0.1 --confirm 127.0.0.1 -- rm -rf /home/nhadmin/outro_dir
check "CRÍTICO: após dry-run + alvo digitado"     0 "exit code: 0"      nh exec 127.0.0.1 --confirm 127.0.0.1 -- rm -rf /home/nhadmin/e2e_dir
check "serviço: reinício validado"                0 "reiniciado e validado" nh service restart 127.0.0.1 nginx --yes
check "serviço: 'sucesso' falso é pego pela validação" 1 "VALIDAÇÃO"    nh service restart 127.0.0.1 frozen --yes
check "serviço: falha de restart é reportada"     1 "falhou"            nh service restart 127.0.0.1 broken --yes
check "serviço inexistente não vira sucesso"      1 "não existe"        nh service restart 127.0.0.1 ghost --yes
check "serviço crítico (ssh) sobe para ALTO"      4 "ALTO"              nh service restart 127.0.0.1 ssh --yes

echo "C) Arquivos (SFTP + SHA-256)"
head -c 200000 /dev/urandom > "$T/payload.bin"
check "upload verificado"                         0 "SHA-256 idêntico"  nh upload 127.0.0.2 "$T/payload.bin" '~/payload.bin' --yes
check "download verificado"                       0 "SHA-256 idêntico"  nh download 127.0.0.2 '~/payload.bin' "$T/back.bin" --yes
cmp -s "$T/payload.bin" "$T/back.bin" && { pass=$((pass+1)); echo "  ok    arquivos idênticos byte a byte"; } || { fail=$((fail+1)); echo "  FALHA arquivos diferem"; }
check "regravar registra a sobrescrita"           0 "sobrescreveu"      nh upload 127.0.0.2 "$T/payload.bin" '~/payload.bin' --yes
check "download não sobrescreve sem --overwrite"  2 "já existe"         nh download 127.0.0.2 '~/payload.bin' "$T/back.bin" --yes
check "upload em /etc exige digitar o alvo"       4 "ALTO"              nh upload 127.0.0.1 "$T/payload.bin" /etc/nh_payload --yes
check "authorized_keys é CRÍTICO"                 4 "dry-run"           nh upload 127.0.0.2 "$T/payload.bin" '~/.ssh/authorized_keys' --confirm 127.0.0.2

echo "D) Workflows"
check "dry-run mostra comandos e requisitos"      0 "requisito.*atendido" nh workflow dry-run restart-service-validated 127.0.0.1 --param service=nginx
check "workflow validado"                         0 "4 etapa\(s\) concluída" nh workflow run restart-service-validated 127.0.0.1 --param service=nginx --yes
check "falha na etapa 2 interrompe as demais"     1 "não executada"     nh workflow run restart-service-validated 127.0.0.1 --param service=ghost --yes
check "sem sudo o workflow é bloqueado"           3 "bloqueado"         nh workflow run restart-service-validated 127.0.0.2 --param service=nginx --yes
check "verificação pré-atualização (GO/NO-GO)"    0 "GO"                nh workflow run pre-update-check 127.0.0.2

echo "E) Evidência e integridade"
check "histórico filtrável"                       0 "service.restart"   nh history --op service --limit 5
check "registros bloqueados também ficam na trilha" 0 "BLOCKED"         nh history --status BLOCKED --limit 3
check "trilha íntegra"                            0 "íntegra"           nh audit verify
python3 - "$NETHUB_DB" <<'PY'
import sqlite3, sys
c = sqlite3.connect(sys.argv[1]); c.execute("UPDATE audit SET status='SUCCESS' WHERE id=(SELECT MIN(id) FROM audit WHERE status='FAILED')"); c.commit()
PY
check "adulteração direta no banco é detectada"   1 "ADULTERAÇÃO"       nh audit verify

echo; echo "Resultado: $pass ok, $fail falha(s)"; rm -f "$NETHUB_DB"
[ "$fail" = 0 ]
