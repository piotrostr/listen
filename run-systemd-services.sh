#!/bin/bash

sudo cp ./target/release/listen /usr/local/bin

sudo cp listener.service /etc/systemd/system/
sudo cp buyer.service /etc/systemd/system/

sudo chmod 644 /etc/systemd/system/listener.service
sudo chmod 644 /etc/systemd/system/buyer.service

sudo systemctl enable listener.service
sudo systemctl start listener.service

sudo systemctl enable buyer.service
sudo systemctl start buyer.service
