#!/usr/bin/env python3
import logging
import os
from datetime import datetime, timezone

from bitwarden_sdk import BitwardenClient, DeviceType, client_settings_from_dict

# Create the BitwardenClient, which is used to interact with the SDK
client = BitwardenClient(
    client_settings_from_dict(
        {
            "apiUrl": os.getenv("API_URL", "http://localhost:4000"),
            "deviceType": DeviceType.SDK,
            "identityUrl": os.getenv("IDENTITY_URL", "http://localhost:33656"),
            "userAgent": "Python",
        }
    )
)

# Add some logging & set the org id
logging.basicConfig(level=logging.DEBUG)
organization_id = os.getenv("ORGANIZATION_ID")

# -- Example Generator Commands --
# Note: using the generator does not require authentication with a server
generated_secret = client.generators().generate() # using default params
very_strong_secret = client.generators().generate(length=128, avoid_ambiguous=False, lowercase=True, uppercase=True, numbers=True, special=True, min_lowercase=2, min_uppercase=2, min_number=4, min_special=4)

print(f"generated secret:   {generated_secret}")
print(f"very strong secret: {very_strong_secret}")

# Set the state file location
# Note: the path must exist, the file will be created & managed by the sdk
state_path = os.getenv("STATE_FILE")

# Attempt to authenticate with the Secrets Manager Access Token
client.auth().login_access_token(os.getenv("ACCESS_TOKEN"), state_path)

# -- Example Project Commands --

project = client.projects().create(organization_id, "ProjectName")
project2 = client.projects().create(organization_id, "AnotherProject")
updated_project = client.projects().update(
    organization_id, project.data.id, "Cool New Project Name"
)
get_that_project = client.projects().get(project.data.id)

input("Press Enter to delete the project...")
client.projects().delete([project.data.id])

print(client.projects().list(organization_id))

# -- Example Secret Commands --

if client.secrets().sync(organization_id, None).data.has_changes is True:
    print("There are changes to sync")
else:
    print("No changes to sync")

last_synced_date = datetime.now(tz=timezone.utc)
print(client.secrets().sync(organization_id, last_synced_date))

secret = client.secrets().create(
    organization_id,
    "TEST_SECRET",
    "This is a test secret",
    "Secret1234!",
    [project2.data.id],
)
secret2 = client.secrets().create(
    organization_id,
    "ANOTHER_SECRET",
    "Secret1234!",
    None,
    [project2.data.id],
)
secret_updated = client.secrets().update(
    organization_id,
    secret.data.id,
    "TEST_SECRET_UPDATED",
    "This as an updated test secret",
    "Secret1234!_updated",
    [project2.data.id],
)
secrets_retrieved = client.secrets().get_by_ids([secret.data.id, secret2.data.id])

# cleanup
input("Press Enter to cleanup secrets and projects...")
client.secrets().delete([secret.id for secret in secrets_retrieved.data.data])

client.projects().delete([project2.data.id])
