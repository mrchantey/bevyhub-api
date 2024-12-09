# bevhub-api

Repo for the bevyhub backend.

## Getting started

1. Populate the local database `just populate`
2. Run: `just run`
If the local packages have changed, call `just populate --force` to repackage them

## Schema changes

If any of the types to be exported, ie `#[derive(TS)]` change, we need to call
`just reschema` which will clear local storage, regenerate bindings and repopulate the database.



## S3
- [dev](https://us-west-2.console.aws.amazon.com/s3/buckets/bevyhub-dev)
- [prod](https://us-west-2.console.aws.amazon.com/s3/buckets/bevhub-prod)

## Mongodb

- 

### Making bucket public

1. disable all `Block Public Access`
2. add policy, changing resource name
	```json
	{
	    "Version": "2012-10-17",
	    "Statement": [
	        {
	            "Sid": "Statement1",
	            "Effect": "Allow",
	            "Principal": "*",
	            "Action": "s3:GetObject",
	            "Resource": "arn:aws:s3:::bevyhub-apps/*"
	        }
	    ]
	}
	```

## Lambda

- [url](https://7sugmdr7lijxtdy4rtyauenuda0abnkl.lambda-url.us-west-2.on.aws/)


### Endpoints
- `/health-check`
- `/scenes`: `Vec<SceneDoc>`
- `/crates/versions/:crate_name`: `Vec<Version>`
- `/crates/unpkg/:crate_name/:version/*path`: `Bytes`
- `/crates/scenes/:crate_name`: `CrateScenes`
- `/crates/scenes/:crate_name/:version`: `CrateScenes`

1. Endpoint: /crates/:crate_name/versions
  URL Example: /bevyhub_template/versions
  Return Type: Json<Vec<Version>>



## Resources

- [cargo lambda](https://www.cargo-lambda.info/guide/getting-started.html)
- [aws lambda rust examples](https://github.com/awslabs/aws-lambda-rust-runtime/blob/main/examples/http-axum/src/main.rs)




## AWS Initial Setup

Here are the steps i took to set up bevyhub on aws:
For all services we use `us-west-2`

### S3


We need two buckets, `bevyhub-dev` and `bevyhub-prod`.
1. [Create Bucket](https://us-west-2.console.aws.amazon.com/s3/bucket/create?region=us-west-2&bucketType=general)
	- default settings

### IAM

The server needs access to s3 buckets so we need to create an iam role:

1. [Create Role](https://us-east-1.console.aws.amazon.com/iam/home?region=us-west-2#/roles/create)
	- Trusted Entity Type: AWS Service
	- Use case: Lambda
	- Permissions
		- AmazonS3FullAccess - buckets
		- AWSLambdaBasicExecutionRole - cloud watch
	- name: bevyhub-lambda
	- description: IAM Role for the Bevyhub rust lambda server.
	- restrict s3 to bevyhub-dev & bevyhub-prod