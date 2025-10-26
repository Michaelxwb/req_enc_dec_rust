#!/usr/bin/env python3
"""
Example Flask application demonstrating the req-enc-dec-rust middleware usage.
This example replicates the functionality from the original Python project.
"""

from flask import Flask, request, jsonify
from req_enc_dec_rust import EncryptionPlugin

app = Flask(__name__)

# Configure the middleware with the same settings as the original
app.config["ENCRYPTION_ALGO"] = "AES"
app.config["ENCRYPTION_SALT"] = b"your_salt_value_1234567890123456"
app.config["ENCRYPTION_KEY"] = b'secret_key_32_bytes_long_1234567890'
app.config["ENCRYPTION_URL_CONFIGS"] = {
    "/api/user": {
        "decrypt_fields": ["email"],
        "encrypt_fields": ["user.token", "user.list.name", "user.list.email.email_name", "user.list.qq"]
    }
}

# Initialize the encryption plugin
EncryptionPlugin(app=app)


@app.route("/api/user", methods=["POST"])
def handle_user():
    """
    Handle user API endpoint that demonstrates encryption/decryption.
    
    The request will have the 'email' field automatically decrypted.
    The response will have the specified fields automatically encrypted.
    """
    request_data = request.get_json()
    print(f"Received email: {request_data.get('email')}")
    
    # Return a complex nested structure that will be partially encrypted
    return {
        "user": {
            "token": "test_token_12345",
            "list": [
                {
                    "name": "test_name01",
                    "email": [
                        {"email_name": "test_email01@example.com"},
                        {"email_name": "test_email02@example.com"}
                    ],
                    "qq": ["test_qq01", "test_qq02"],
                },
                {
                    "name": "test_name02",
                    "email": [],
                    "qq": [],
                }
            ]
        }
    }


@app.route("/api/health", methods=["GET"])
def health_check():
    """Health check endpoint - no encryption/decryption applied"""
    return {"status": "healthy", "service": "req-enc-dec-rust"}


@app.route("/api/test-encryption", methods=["POST"])
def test_encryption():
    """Test endpoint to manually test encryption/decryption"""
    data = request.get_json()
    
    # Manual encryption/decryption for testing using same config as middleware
    from req_enc_dec_rust import AESCipher
    import hashlib
    
    # Use the same key and salt as the middleware
    key = app.config["ENCRYPTION_KEY"]
    salt = app.config["ENCRYPTION_SALT"]
    
    # Create cipher with same configuration as middleware
    key_hash = hashlib.sha256(key).digest()
    iv = salt[:16]  # Use first 16 bytes of salt as IV
    cipher = AESCipher(key_hash[:32], iv)
    
    if 'encrypt' in data:
        encrypted = cipher.encrypt(data['encrypt'])
        return {"encrypted": encrypted}
    elif 'decrypt' in data:
        decrypted = cipher.decrypt(data['decrypt'])
        return {"decrypted": decrypted}
    else:
        return {"error": "Provide 'encrypt' or 'decrypt' field"}


if __name__ == "__main__":
    print("Starting Flask application with req-enc-dec-rust middleware...")
    print("Test the /api/user endpoint with POST request containing encrypted email field")
    print("Example request: POST /api/user with JSON: {'email': 'encrypted_email_value'}")
    app.run(host="0.0.0.0", port=5001, debug=True)