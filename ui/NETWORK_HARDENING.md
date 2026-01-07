# Path and File Name : /home/ransomeye/rebuild/ui/NETWORK_HARDENING.md
# Author: nXxBku0CKFAJCBN3X1g3bQk7OxYQylg8CMw1iGsq7gU
# Details: Network hardening configuration guide for Windows network access

# RansomEye UI - Network Hardening Guide

This document explains how to securely configure the RansomEye UI for network access from Windows systems.

## Environment Variables

### Server Binding

- **`RANSOMEYE_UI_BIND_ADDRESS`** (default: `127.0.0.1`)
  - Bind address for the UI server
  - Safe values:
    - `127.0.0.1` or `localhost` - localhost only (safest, default)
    - `0.0.0.0` - all network interfaces (use with firewall rules)
    - Specific IP (e.g., `192.168.1.100`) - bind to specific interface
  - **Security Note**: Binding to `0.0.0.0` exposes the UI to all network interfaces. Ensure firewall rules are configured.

- **`RANSOMEYE_UI_BIND_PORT`** (default: `8081`)
  - Port number for the UI server (1-65535)
  - Legacy: `RANSOMEYE_UI_PORT` is also supported

### CORS Configuration

- **`RANSOMEYE_UI_ALLOWED_ORIGINS`** (default: empty - same-origin only)
  - Comma-separated list of allowed origins for CORS
  - Example: `http://192.168.1.50:8081,https://ransomeye.example.com`
  - **Security**: No wildcard (`*`) allowed. Only explicit origins are permitted.
  - **Methods**: Only `GET` and `HEAD` are allowed by default.

- **`RANSOMEYE_UI_CORS_CREDENTIALS`** (default: `false`)
  - Enable CORS credentials (cookies, authorization headers)
  - Set to `true` only if explicitly required
  - **Security**: Disabled by default for security

### Proxy Configuration

- **`RANSOMEYE_UI_TRUST_PROXY`** (default: `false`)
  - Trust `X-Forwarded-For` and `X-Forwarded-Proto` headers
  - Set to `true` only when behind a trusted reverse proxy (nginx, Apache, etc.)
  - **Security**: Disabled by default to prevent header spoofing

### Security Headers

- **`RANSOMEYE_UI_X_FRAME_OPTIONS`** (default: `DENY`)
  - X-Frame-Options header value
  - Options: `DENY` (recommended) or `SAMEORIGIN`

- **`RANSOMEYE_UI_REFERRER_POLICY`** (default: `strict-origin-when-cross-origin`)
  - Referrer-Policy header value
  - Controls referrer information sent with requests

- **`RANSOMEYE_UI_CSP`** (default: safe default with inline scripts allowed)
  - Content-Security-Policy header
  - Modify only if you understand CSP and need custom policies

## Accessing UI from Windows

### Direct Access (No Proxy)

1. **Configure bind address** (in `/etc/ransomeye/ui.env` or systemd service):
   ```bash
   RANSOMEYE_UI_BIND_ADDRESS=0.0.0.0
   RANSOMEYE_UI_BIND_PORT=8081
   ```

2. **Configure CORS** (if accessing from different origin):
   ```bash
   RANSOMEYE_UI_ALLOWED_ORIGINS=http://192.168.1.50:8081
   ```
   Replace `192.168.1.50` with your Windows machine's IP address.

3. **Configure firewall** (on Linux server):
   ```bash
   sudo ufw allow 8081/tcp
   # Or more restrictive:
   sudo ufw allow from 192.168.1.0/24 to any port 8081
   ```

4. **Access from Windows browser**:
   ```
   http://<server-ip>:8081
   ```
   Example: `http://192.168.1.100:8081`

### Reverse Proxy Access (Recommended for Production)

1. **Configure nginx/Apache** as reverse proxy:
   ```nginx
   server {
       listen 80;
       server_name ransomeye.example.com;
       
       location / {
           proxy_pass http://127.0.0.1:8081;
           proxy_set_header Host $host;
           proxy_set_header X-Real-IP $remote_addr;
           proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
           proxy_set_header X-Forwarded-Proto $scheme;
       }
   }
   ```

2. **Configure RansomEye UI**:
   ```bash
   RANSOMEYE_UI_BIND_ADDRESS=127.0.0.1  # Only listen on localhost
   RANSOMEYE_UI_TRUST_PROXY=true        # Trust proxy headers
   RANSOMEYE_UI_ALLOWED_ORIGINS=https://ransomeye.example.com
   ```

3. **Access from Windows**:
   ```
   https://ransomeye.example.com
   ```

## Security Best Practices

### 1. Firewall Rules
- Always configure firewall rules when binding to `0.0.0.0`
- Restrict access to specific IP ranges when possible
- Example: `sudo ufw allow from 192.168.1.0/24 to any port 8081`

### 2. CORS Configuration
- Never use wildcard (`*`) origins
- Only allow origins you explicitly trust
- Keep credentials disabled unless required

### 3. Proxy Trust
- Only enable `RANSOMEYE_UI_TRUST_PROXY=true` when behind a trusted reverse proxy
- Verify proxy is properly configured and trusted
- Disable in direct access scenarios

### 4. Bind Address
- Use `127.0.0.1` for localhost-only access (safest)
- Use specific interface IP when possible
- Use `0.0.0.0` only with proper firewall rules

### 5. HTTPS (Production)
- Always use HTTPS in production
- Configure reverse proxy with SSL/TLS certificates
- Use Let's Encrypt or enterprise certificates

## Troubleshooting

### Cannot Access from Windows

1. **Check bind address**:
   ```bash
   # Verify server is listening on correct interface
   sudo netstat -tlnp | grep 8081
   ```

2. **Check firewall**:
   ```bash
   # Verify firewall allows connections
   sudo ufw status
   ```

3. **Check CORS**:
   - Verify `RANSOMEYE_UI_ALLOWED_ORIGINS` includes your Windows machine's origin
   - Check browser console for CORS errors

4. **Check proxy trust**:
   - If behind proxy, ensure `RANSOMEYE_UI_TRUST_PROXY=true`
   - Verify proxy headers are correctly set

### CORS Errors in Browser

- Verify `RANSOMEYE_UI_ALLOWED_ORIGINS` includes the exact origin (protocol, host, port)
- Example: `http://192.168.1.50:8081` (not `http://192.168.1.50`)

### Method Not Allowed (405)

- Verify you're using correct HTTP methods
- Dashboard/data endpoints are GET-only
- State-changing operations require POST/DELETE

## Validation

On server startup, the following information is logged:
- Bind address and port
- CORS allowed origins
- CORS credentials setting
- Proxy trust setting
- Security warnings (if applicable)

Check logs with:
```bash
sudo journalctl -u ransomeye-ui -f
```

## Example Configuration Files

### `/etc/ransomeye/ui.env` (Direct Access)
```bash
RANSOMEYE_UI_BIND_ADDRESS=0.0.0.0
RANSOMEYE_UI_BIND_PORT=8081
RANSOMEYE_UI_ALLOWED_ORIGINS=http://192.168.1.50:8081,http://192.168.1.51:8081
RANSOMEYE_UI_CORS_CREDENTIALS=false
RANSOMEYE_UI_TRUST_PROXY=false
```

### `/etc/ransomeye/ui.env` (Reverse Proxy)
```bash
RANSOMEYE_UI_BIND_ADDRESS=127.0.0.1
RANSOMEYE_UI_BIND_PORT=8081
RANSOMEYE_UI_ALLOWED_ORIGINS=https://ransomeye.example.com
RANSOMEYE_UI_CORS_CREDENTIALS=false
RANSOMEYE_UI_TRUST_PROXY=true
```

---

**© RansomEye.Tech | Support: Gagan@RansomEye.Tech**

