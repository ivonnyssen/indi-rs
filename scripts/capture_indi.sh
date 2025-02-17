#!/bin/bash

# Capture INDI traffic on port 7624
echo "Starting INDI traffic capture..."
sudo tcpdump -i lo -w indi_capture.pcap 'port 7624' -v

# Note: Run this in another terminal to stop capture:
# sudo pkill -f "tcpdump.*port 7624"
