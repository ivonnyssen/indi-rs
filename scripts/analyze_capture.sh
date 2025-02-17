#!/bin/bash

# Convert the pcap to text and extract the XML content
echo "Analyzing INDI traffic..."
tcpdump -r indi_capture.pcap -A 2>/dev/null | grep -A1 "<.*>" | grep -v "^--$" > indi_messages.txt

echo "INDI messages have been saved to indi_messages.txt"
echo "Here are the unique message types found:"
grep "<.*>" indi_messages.txt | sort -u
