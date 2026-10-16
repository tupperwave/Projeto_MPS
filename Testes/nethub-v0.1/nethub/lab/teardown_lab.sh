#!/bin/bash
# Remove o laboratório criado por setup_lab.sh
[ "$(id -u)" = 0 ] || { echo "execute como root"; exit 1; }
L=/tmp/nhlab
[ -f $L/sshd.pid ] && kill "$(cat $L/sshd.pid)" 2>/dev/null
[ -f $L/http.pid ] && kill "$(cat $L/http.pid)" 2>/dev/null
for u in nhadmin nhuser nokey; do userdel -r "$u" 2>/dev/null; done
rm -f /etc/sudoers.d/nhlab /usr/local/bin/systemctl
rm -rf /var/lib/fakesd /run/systemd/system $L
echo "Laboratório removido."
