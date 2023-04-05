from minio import Minio
from minio.error import S3Error
import os
from time import sleep

def recursive_upload(client, directory):
    for sub in os.listdir(directory):
        sub = f'{directory}/{sub}'
        if os.path.isdir(sub):
            if sub[-1] in ('A','B','C','D','E', 'G'):
                recursive_upload(client, sub)
        else:
            client.fput_object(
                "devcade", sub.lstrip('./'), sub.lstrip('./'),
            )

def main():
    sleep(1)
    # Create a client with the MinIO server playground, its access key
    # and secret key.
    client = Minio(
        "minio:9000",
        access_key="DEVCADE1234",
        secret_key="DEVCADE1234",
        secure=False
    )

    # Make 'asiatrip' bucket if not exist.
    found = client.bucket_exists("devcade")
    if not found:
        client.make_bucket("devcade")
    else:
        print("Bucket 'devcade' already exists")
    os.chdir('./TESTING/data')
    recursive_upload(client, '.')


if __name__ == "__main__":
    try:
        main()
    except S3Error as exc:
        print("error occurred.", exc)