import boto3
from botocore.exceptions import ClientError
import json

secret_name = "bvh_to_workflow_job"
region_name = "us-east-1"

session = boto3.session.Session()
client = session.client(
    service_name='secretsmanager',
    region_name=region_name
)

secret_response = client.get_secret_value(
    SecretId=secret_name
)

secret_string = secret_response["SecretString"]
secrets = json.loads(secret_string)

with open("secrets-rendered.env", "w") as file:
  for k, v in secrets.items():
    file.write(f"{k}={v}\n")
  
