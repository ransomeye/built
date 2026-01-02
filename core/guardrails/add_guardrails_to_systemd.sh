#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/core/guardrails/add_guardrails_to_systemd.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Adds ExecStartPre guardrails enforcement to all systemd services

set -euo pipefail

SYSTEMD_DIR="/home/ransomeye/rebuild/systemd"

if [[ ! -d "$SYSTEMD_DIR" ]]; then
    echo "Error: Systemd directory not found: $SYSTEMD_DIR"
    exit 1
fi

for service_file in "$SYSTEMD_DIR"/*.service; do
    if [[ ! -f "$service_file" ]]; then
        continue
    fi
    
    service_name=$(basename "$service_file" .service)
    
    # Skip if ExecStartPre already exists
    if grep -q "ExecStartPre.*ransomeye-guardrails" "$service_file"; then
        echo "Skipping $service_name (already has guardrails)"
        continue
    fi
    
    # Find ExecStart line
    if grep -q "^ExecStart=" "$service_file"; then
        # Add ExecStartPre before ExecStart
        sed -i "/^ExecStart=/i ExecStartPre=/usr/bin/ransomeye-guardrails enforce --context service --data $service_name" "$service_file"
        echo "Added guardrails to $service_name"
    else
        echo "Warning: No ExecStart found in $service_name"
    fi
done

echo "✓ Guardrails enforcement added to all systemd services"

