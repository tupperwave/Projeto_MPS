#!/bin/bash
# Laboratório local do NetHub (USE APENAS EM VM/CONTÊINER DESCARTÁVEL — cria usuários e sudo sem senha).
# Cria 4 "hosts" em loopback, todos testáveis sem rede externa:
#   127.0.0.1  nhadmin  SSH + sudo sem senha        → capacidade AUTORIZADA
#   127.0.0.2  nhuser   SSH sem sudo                → AUTENTICADA (somente leitura)
#   127.0.0.3  nokey    SSH sem chave autorizada    → apenas ALCANÇÁVEL
#   127.0.0.4  (sem SSH, só HTTP :8080)             → nenhuma ação disponível (RN-04)
# O sshd escuta na porta 2222. systemd é emulado por lab/fake-systemctl.
set -euo pipefail
[ "$(id -u)" = 0 ] || { echo "execute como root"; exit 1; }
HERE="$(cd "$(dirname "$0")" && pwd)"
L=/tmp/nhlab; mkdir -p "$L" /run/sshd /var/lib/fakesd /run/systemd/system

for u in nhadmin nhuser nokey; do
  id "$u" >/dev/null 2>&1 || useradd -m -s /bin/bash "$u"
  usermod -p '*' "$u"
done
echo 'nhadmin ALL=(ALL) NOPASSWD:ALL' > /etc/sudoers.d/nhlab && chmod 440 /etc/sudoers.d/nhlab

[ -f "$L/operator_key" ] || ssh-keygen -q -t ed25519 -N '' -f "$L/operator_key"
for u in nhadmin nhuser; do
  install -d -m 700 -o "$u" -g "$u" "/home/$u/.ssh"
  install -m 600 -o "$u" -g "$u" "$L/operator_key.pub" "/home/$u/.ssh/authorized_keys"
done

[ -f "$L/hostkey" ] || ssh-keygen -q -t ed25519 -N '' -f "$L/hostkey"
cat > "$L/sshd_config" <<CFG
Port 2222
ListenAddress 127.0.0.1
ListenAddress 127.0.0.2
ListenAddress 127.0.0.3
HostKey $L/hostkey
PidFile $L/sshd.pid
PasswordAuthentication no
KbdInteractiveAuthentication no
PubkeyAuthentication yes
UsePAM no
AllowUsers nhadmin nhuser nokey
Subsystem sftp /usr/lib/openssh/sftp-server
LogLevel ERROR
CFG
if [ -f "$L/sshd.pid" ] && kill -0 "$(cat "$L/sshd.pid")" 2>/dev/null; then kill "$(cat "$L/sshd.pid")"; sleep 0.5; fi
/usr/sbin/sshd -f "$L/sshd_config"

install -m 755 "$HERE/fake-systemctl" /usr/local/bin/systemctl
rm -f /var/lib/fakesd/*.state
mk() { printf 'LoadState=loaded\nActiveState=%s\nSubState=%s\nMainPID=%s\nActiveEnterTimestampMonotonic=%s\nDesc=%s\n%s\n' "$2" "$3" "$4" "$5" "$6" "${7:-}" > "/var/lib/fakesd/$1.state"; }
mk nginx      active   running 811 1000 "A high performance web server"
mk cron       active   running 622 900  "Regular background program processing daemon"
mk postgresql active   running 933 1100 "PostgreSQL RDBMS"
mk ssh        active   running 501 500  "OpenBSD Secure Shell server"
mk cups       inactive dead    0   0    "CUPS Scheduler"
mk broken     active   running 700 1200 "Serviço que falha ao reiniciar" "FailRestart=1"
mk frozen     active   running 710 1300 "Serviço cujo restart não faz nada" "Frozen=1"

if [ -f "$L/http.pid" ] && kill -0 "$(cat "$L/http.pid")" 2>/dev/null; then kill "$(cat "$L/http.pid")"; fi
(cd "$L" && setsid nohup python3 -m http.server 8080 --bind 127.0.0.4 </dev/null >/dev/null 2>&1 & echo $! > "$L/http.pid")
sleep 0.7
echo "Laboratório pronto. Chave do operador: $L/operator_key"
echo "Teste: ssh -i $L/operator_key -p 2222 nhadmin@127.0.0.1 id"
