#!/bin/bash
mkdir -p /root/.termlink/secrets
echo -n "4f7860f1953c8c3fa06d861fb4729f09c8329aed9795e4950ed8292d8e601932" > /root/.termlink/secrets/ring20-management.hex
sed -i 's/\[hubs\.proxmox4\]/[hubs.ring20-management]/;s/192.168.10.122/192.168.10.109/;s/proxmox4\.hex/ring20-management.hex/' /root/.termlink/hubs.toml
echo "Done. Profile ring20-management → 192.168.10.109:9100"
cat /root/.termlink/hubs.toml
