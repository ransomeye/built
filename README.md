# RansomEye — Enterprise Security Platform

**RansomEye** is a comprehensive, enterprise-grade cybersecurity platform designed to detect, analyze, and respond to ransomware and advanced persistent threats (APTs). Built with 23 integrated phases, RansomEye provides end-to-end security orchestration, automated response, and AI-powered threat intelligence.

## 🏗️ Architecture

RansomEye is organized into 23 distinct phases, each serving a critical security function:

### Core Modules
- **Phase 1**: Core Engine & Installer
- **Phase 2**: AI Core & Model Registry
- **Phase 3**: Alert Engine & Policy Manager
- **Phase 4**: KillChain & Forensic Dump
- **Phase 5**: LLM Summarizer
- **Phase 6**: Incident Response & Playbooks
- **Phase 7**: SOC Copilot
- **Phase 8**: Threat Correlation Engine
- **Phase 9**: Network Scanner
- **Phase 10**: DB Core (PostgreSQL)
- **Phase 11**: UI & Dashboards
- **Phase 12**: Orchestrator (Master Flow)

### Advanced Modules
- **Phase 13**: Forensic Engine (Advanced)
- **Phase 14**: LLM Behavior Summarizer (Expanded)
- **Phase 15**: SOC Copilot (Advanced)
- **Phase 16**: Deception Framework
- **Phase 17**: AI Assistant (Governor Mode)
- **Phase 18**: Threat Intelligence Feed Engine
- **Phase 19**: HNMP Engine
- **Phase 20**: Global Validator

### Standalone Agents
- **Phase 21**: Linux Agent
- **Phase 22**: Windows Agent
- **Phase 23**: DPI Probe

### Supporting Infrastructure
- **Guardrails**: Fail-closed security enforcement
- **Retention**: Data retention and disk management
- **Trust**: Cryptographic signing and verification

## 🚀 Quick Start

### Prerequisites
- Python 3.8+
- PostgreSQL 12+
- Linux (Ubuntu 20.04+ recommended)
- Systemd (for service management)
- Root/sudo privileges for installation

### Installation

**IMPORTANT: The ONLY supported installation method is via the root-level `install.sh` script.**

```bash
# Clone the repository
git clone https://github.com/yourusername/ransomeye-rebuild.git
cd ransomeye-rebuild

# Run the installer (ONLY supported method)
sudo ./install.sh

# The installer will:
# 1. Display and require EULA acceptance
# 2. Install the core RansomEye stack
# 3. Optionally install standalone modules (DPI Probe, Linux Agent, Windows Agent)
# 4. Run post-install validation automatically
```

**Unsupported Installation Methods:**
- ❌ Running individual module installers directly
- ❌ Manual installation steps
- ❌ Using `python3 -m ransomeye_installer.installer` directly
- ❌ Using `ransomeye_operations install` directly

**Uninstallation:**

```bash
# Uninstall RansomEye (ONLY supported method)
sudo ./uninstall.sh [--preserve-logs] [--preserve-evidence]
```

The uninstaller will:
- Stop all services cleanly
- Uninstall in reverse dependency order (standalone modules first, then core)
- Optionally preserve logs and evidence
- Remove all installation artifacts without leaving orphaned files

**Manual Verification (if needed):**

```bash
# Run post-install validator manually
sudo ./post_install_validator.py
```

**Note:** Post-install validation runs automatically at the end of `install.sh`. Manual execution is only needed for troubleshooting.

### Configuration

1. Copy environment template:
```bash
cp .env.example .env
```

2. Configure database and API settings in `.env`

3. Start services:
```bash
sudo systemctl start ransomeye-master-core
sudo systemctl status ransomeye-*
```

## 📋 Features

- **AI-Powered Detection**: Machine learning models with SHAP explainability
- **Offline Operation**: Fully air-gapped capable, no internet dependencies
- **Real-time Monitoring**: Continuous threat detection and alerting
- **Automated Response**: Playbook-driven incident response
- **Forensic Analysis**: Complete kill chain reconstruction
- **Threat Intelligence**: Integrated feeds from MISP, OTX, Talos, ThreatFox
- **Compliance**: Built-in compliance scanning and reporting
- **Multi-Format Exports**: PDF, HTML, CSV reporting

## 🔒 Security

- **Fail-Closed Architecture**: System fails securely on violations
- **Cryptographic Signing**: All artifacts are signed and verified
- **Encrypted Storage**: Database encryption for sensitive data
- **No Hardcoded Secrets**: All configuration via environment variables
- **Rootless Operation**: Services run with minimal privileges

## 📊 Data Management

- **PostgreSQL Database**: Centralized data storage
- **7-Year Retention**: Configurable data retention policies
- **Automatic Cleanup**: Disk space management when >80% full
- **Partitioned Tables**: Optimized for large-scale data

## 🤖 AI & Machine Learning

- **Model Registry**: Centralized model management
- **SHAP Explainability**: All ML outputs include explainability
- **Offline Models**: Pre-trained models included (via Git LFS)
- **Adversarial Defense**: Built-in model robustness

## 📝 Reporting

All modules export reports in multiple formats:
- **PDF**: Formatted reports with branding
- **HTML**: Interactive web reports
- **CSV**: Machine-readable data exports
- **JSON**: Structured data (optional)

All reports include:
- Timestamp and build hash
- Model version information
- SHAP explainability context
- Footer: "© RansomEye.Tech | Support: Gagan@RansomEye.Tech"

## 🧪 Testing

```bash
# Run all tests
python -m pytest tests/

# Run specific module tests
python -m pytest ransomeye_guardrails/tests/

# Validate guardrails
python -m ransomeye_guardrails.scanner
```

## 📚 Documentation

- **Installation Guide**: See `install.sh` and `post_install_validator.py`
- **Uninstallation Guide**: See `uninstall.sh`
- **API Documentation**: See individual module READMEs
- **Architecture**: See project specification in user rules

## 🔧 Installation & Uninstallation

**SUPPORTED METHODS (ONLY):**

1. **Installation**: `sudo ./install.sh`
   - This is the ONLY supported installation method
   - All other installation methods are unsupported

2. **Uninstallation**: `sudo ./uninstall.sh [--preserve-logs] [--preserve-evidence]`
   - This is the ONLY supported uninstallation method
   - All other uninstallation methods are unsupported

**UNSUPPORTED METHODS:**

- Direct execution of module installers
- Manual installation procedures
- Direct Python module execution
- Direct Rust binary execution
- Partial installations

**For production deployments, ALWAYS use the root-level `install.sh` and `uninstall.sh` scripts.**

## 🔧 Development

### Project Structure
```
/home/ransomeye/rebuild/
├── install.sh                 # Unified installer
├── uninstall.sh              # Unified uninstaller
├── requirements.txt          # Unified dependencies
├── systemd/                 # All systemd service files
├── logs/                    # Centralized logging
├── ransomeye_*/            # Individual phase modules
└── tests/                   # Test suites
```

### Code Standards
- All files must include mandatory header
- No hardcoded IPs, ports, or credentials
- Environment variables for all configuration
- SHAP explainability for all ML outputs
- Comprehensive test coverage

## 🔄 GitHub Synchronization

RansomEye includes automatic GitHub synchronization that runs every hour by default.

### Initial Setup

1. **Set up Git credentials** (one-time):
   ```bash
   ./setup_git_credentials.sh
   ```
   Choose between credential store or SSH authentication.

2. **Enable auto-sync**:
   ```bash
   sudo ./setup_auto_sync.sh
   ```

### Manual Sync

```bash
# Sync manually
./github_auto_sync.sh

# Or use the original sync script
./sync_to_github.sh
```

### Auto-Sync Management

```bash
# Check timer status
sudo systemctl status ransomeye-github-sync.timer

# View sync logs
sudo journalctl -u ransomeye-github-sync.service -f

# Check log file
tail -f logs/github_sync.log

# Manually trigger sync
sudo systemctl start ransomeye-github-sync.service

# Disable auto-sync
sudo systemctl stop ransomeye-github-sync.timer

# Enable auto-sync
sudo systemctl start ransomeye-github-sync.timer
```

### Features

- **Automatic**: Syncs every hour via systemd timer
- **Safe**: Only pushes if there are changes
- **Logged**: All operations logged to `logs/github_sync.log`
- **Lock-protected**: Prevents concurrent sync operations
- **Git LFS**: Automatically handles large model and dataset files

## 📞 Support

**Support Email**: Gagan@RansomEye.Tech

## 📄 License

Proprietary - RansomEye.Tech

## 🙏 Acknowledgments

Built with enterprise-excellent standards, designed to outperform leading cybersecurity solutions.

---

**© RansomEye.Tech | Support: Gagan@RansomEye.Tech**

