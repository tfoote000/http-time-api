#!/bin/bash
set -e

# Generate self-signed TLS certificate for development/testing
# For production, use a proper CA-signed certificate (e.g., Let's Encrypt)

echo "=== Generating Self-Signed TLS Certificate for Development ==="

# Output directory
OUTPUT_DIR="$(dirname "$0")"
cd "$OUTPUT_DIR"

# Certificate details
DOMAIN="localhost"
DAYS=365

echo ""
echo "Domain: $DOMAIN"
echo "Valid for: $DAYS days"
echo "Output directory: $OUTPUT_DIR"
echo ""

# Generate private key
echo "Generating private key..."
openssl genrsa -out key.pem 2048

# Generate certificate signing request
echo "Generating certificate signing request..."
openssl req -new -key key.pem -out csr.pem -subj "/CN=$DOMAIN"

# Generate self-signed certificate
echo "Generating self-signed certificate..."
openssl x509 -req -days $DAYS -in csr.pem -signkey key.pem -out cert.pem

# Clean up CSR
rm csr.pem

# Set permissions
chmod 600 key.pem
chmod 644 cert.pem

echo ""
echo "✓ Certificate generated successfully!"
echo ""
echo "Files created:"
echo "  - cert.pem (certificate)"
echo "  - key.pem (private key)"
echo ""
echo "Usage:"
echo "  TLS_CERT_PATH=$OUTPUT_DIR/cert.pem"
echo "  TLS_KEY_PATH=$OUTPUT_DIR/key.pem"
echo ""
echo "WARNING: This is a self-signed certificate for development only!"
echo "         Browsers will show security warnings."
echo "         For production, use a CA-signed certificate."
