# NOTE - for example purpose only - import gem instead
require 'bitwarden-sdk-secrets'

token = ENV['ACCESS_TOKEN']
state_file = ENV['STATE_FILE']

# Configuring the URLS is optional, set them to nil to use the default values
api_url = ENV['API_URL']
identity_url = ENV['IDENTITY_URL']

bitwarden_settings = BitwardenSDKSecrets::BitwardenSettings.new(api_url, identity_url)

bw_client = BitwardenSDKSecrets::BitwardenClient.new(bitwarden_settings)
response = bw_client.auth.login_access_token(token, state_file)
puts response

# CREATE project
project_name = 'Test project 1'
response = bw_client.projects.create(project_name)
puts response
project_id = response['id']

# GET project
response = bw_client.projects.get(project_id)
puts response

# LIST projects
response = bw_client.projects.list
puts response

# UPDATE projects
name = 'Updated test project 1'
response = bw_client.projects.update(project_id, name)
puts response

# CREATE secret
key = 'AWS-SES'
note = 'Private account'
value = '8t27.dfj;'
response = bw_client.secrets.create(key, value, note, [project_id])
puts response
secret_id = response['id']

# GET secret
response = bw_client.secrets.get(secret_id)
puts response

# GET secret by ids
response = bw_client.secrets.get_by_ids([secret_id])
puts response

# LIST secrets
response = bw_client.secrets.list
puts response

# SYNC secrets
response = bw_client.secrets.sync(nil)
last_synced_date = Time.now.utc.strftime('%Y-%m-%dT%H:%M:%S.%6NZ')
puts response

response = bw_client.secrets.sync(last_synced_date)
puts response

# UPDATE secret
note = 'updated password'
value = '7I.ert10AjK'
response = bw_client.secrets.update(secret_id, key, value, note, [project_id])
puts response

# DELETE secret
response = bw_client.secrets.delete([secret_id])
puts response

# DELETE project
response = bw_client.projects.delete([project_id])
puts response
