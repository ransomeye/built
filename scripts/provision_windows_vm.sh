#!/bin/bash
# Path and File Name : /home/ransomeye/rebuild/scripts/provision_windows_vm.sh
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details of functionality of this file: Windows VM Provisioning Script - Provisions Windows 10/11 VM using QEMU/KVM

set -euo pipefail

VM_NAME="ransomeye-windows-test"
VM_DISK="/var/lib/libvirt/images/${VM_NAME}.qcow2"
VM_RAM="4096"
VM_CPUS="2"
WINDOWS_ISO="${WINDOWS_ISO:-/tmp/windows.iso}"

echo "=== Windows VM Provisioning Script ==="
echo "VM Name: $VM_NAME"
echo ""

# Check for virtualization support
if ! grep -q vmx /proc/cpuinfo && ! grep -q svm /proc/cpuinfo; then
    echo "ERROR: CPU does not support virtualization"
    exit 1
fi

# Check for KVM
if [ ! -e /dev/kvm ]; then
    echo "ERROR: /dev/kvm not found. Install KVM support."
    exit 1
fi

# Check for virt-manager/virt-install
if ! command -v virt-install >/dev/null 2>&1; then
    echo "Installing virt-install..."
    sudo apt-get update
    sudo apt-get install -y virt-manager qemu-kvm libvirt-daemon-system libvirt-clients bridge-utils
    sudo systemctl enable libvirtd
    sudo systemctl start libvirtd
    sudo usermod -aG libvirt $USER
    echo "Please log out and back in for group changes to take effect"
fi

# Create VM disk if it doesn't exist
if [ ! -f "$VM_DISK" ]; then
    echo "Creating VM disk: $VM_DISK (40GB)"
    sudo qemu-img create -f qcow2 "$VM_DISK" 40G
fi

# Check if Windows ISO exists
if [ ! -f "$WINDOWS_ISO" ]; then
    echo "WARNING: Windows ISO not found at $WINDOWS_ISO"
    echo "Please download Windows 10/11 ISO and set WINDOWS_ISO environment variable"
    echo "Example: export WINDOWS_ISO=/path/to/windows.iso"
    exit 1
fi

# Create VM
echo "Creating Windows VM..."
sudo virt-install \
    --name "$VM_NAME" \
    --ram "$VM_RAM" \
    --vcpus "$VM_CPUS" \
    --disk path="$VM_DISK",size=40,format=qcow2 \
    --cdrom "$WINDOWS_ISO" \
    --network network=default \
    --graphics vnc,listen=0.0.0.0 \
    --noautoconsole \
    --os-type windows \
    --os-variant win10

echo ""
echo "VM created. Connect via VNC to complete Windows installation."
echo "VNC port: Check with 'sudo virsh vncdisplay $VM_NAME'"
echo ""
echo "After Windows installation:"
echo "1. Enable ETW (Event Tracing for Windows)"
echo "2. Enable test signing (bcdedit /set testsigning on)"
echo "3. Install Windows Agent"
echo "4. Execute tests"

