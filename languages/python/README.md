# Build locally
## Requirements

- Python 3
- Rust
- `maturin` (install with `pip install maturin`, or `pip install maturin[patchelf]` on Linux)
- `npm`
- `uv` (recommended; used in test automation)

## Build

```bash
npm install
npm run schemas # generate schemas.py

cd languages/python/
```

You will need to build and run the script using a virtual environment.
This will be slightly different depending on the OS you are using:

```bash
# --- Linux/macOS ---
python3 -m venv .venv
source .venv/bin/activate

# --- Windows ---
python -m venv venv

venv\Scripts\activate.bat # cmd.exe
venv\Scripts\Activate.ps1 # Powershell
```

## Run

```bash
maturin develop
python3 ./example.py

deactivate # run this to close the virtual session
```

You can now import `BitwardenClient` in your Python code with:
```python
from bitwarden_sdk import BitwardenClient
```

# Use without building locally

```bash
pip install bitwarden-sdk
```

# Run

Set the `ORGANIZATION_ID` and `ACCESS_TOKEN` environment variables to your organization ID and access token, respectively.

```bash
python3 ./example.py
```
