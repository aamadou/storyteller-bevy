sudo cp x11docker-inferencejob.service /etc/systemd/system
sudo systemctl daemon-reload
sudo systemctl enable x11docker-inferencejob.service
sudo systemctl start x11docker-inferencejob.service
