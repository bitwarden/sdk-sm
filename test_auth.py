#!/usr/bin/env python3
"""Simple test to debug authentication issue"""

import os
import sys
sys.path.insert(0, '/Users/daniel/Workspace/sdk-sm/languages/python')

from bitwarden_sdk import BitwardenClient, DeviceType, client_settings_from_dict

# Set up client
client_settings = client_settings_from_dict({
    "api_url": os.getenv("API_URL", "http://localhost:4000"),
    "identity_url": os.getenv("IDENTITY_URL", "http://localhost:4000"),
    "user_agent": "Python SDK Test",
    "device_type": DeviceType.SDK
})

client = BitwardenClient(client_settings)

# Try to authenticate
access_token = "0.ec2c1d46-6a4b-4751-a310-af9601317f2d.C2IgxjjLF7qSshsbwe8JGcbM075YXw:X8vbvA0bduihIDe/qrzIQQ=="
print(f"Attempting auth with token: {access_token[:50]}...")
print(f"API URL: {os.getenv('API_URL', 'http://localhost:4000')}")
print(f"Identity URL: {os.getenv('IDENTITY_URL', 'http://localhost:4000')}")

try:
    client.auth().login_access_token(access_token, None)
    print("✅ Authentication successful!")
except Exception as e:
    print(f"❌ Authentication failed: {e}")
    import traceback
    traceback.print_exc()