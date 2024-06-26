#!/bin/bash
dnf update -y

# install docker
dnf install -y docker
service docker start
usermod -a -G docker ec2-user

# create daemon
service_name="minecraft-server"
filename="/etc/systemd/system/${service_name}.service"

cat <<EOF >${filename}
[Unit]
Description=Minecraft Server
After=docker.service
Requires=docker.service

[Service]
Restart=always
ExecStart=/usr/bin/docker run --rm --name minecraft-server -p 25565:25565 -e EULA=TRUE -e TYPE=PAPER -e VERSION=1.20.6 -e MEMORY=4G -v /var/minecraft:/data itzg/minecraft-server:latest
ExecStop=/usr/bin/docker stop minecraft-server

[Install]
WantedBy=multi-user.target
EOF
chmod 644 $filename

systemctl daemon-reload
systemctl enable minecraft-server.service
systemctl start minecraft-server.service