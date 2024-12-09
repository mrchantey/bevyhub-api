set windows-shell := ["C:/tools/cygwin/bin/sh.exe","-c"]
set dotenv-load

default:
	just --list --unsorted

init-repo:
	just populate --force
	just bindings

# Oregon
region:="us-west-2"

e2e *args:
	cargo test --test mongo_sets_latest {{args}}

test *args:
	cargo test -p bevyhub_api --lib -- {{args}}
	cargo test -p cli -- {{args}}

test-w *args:
	just watch just test {{args}}


test-curl *args:
	cargo run --example curl {{args}}
test-curl-prod *args:
	API_ENV=prod cargo run --example curl {{args}} 


test-cli *args:
	just watch cargo test -p cli -- {{args}}

example name *args:
	cargo run --example {{name}} {{args}}

run:
	just watch 'cargo run --example server'

cli *args:
	cargo run -p cli -- {{args}}

lambda:
	cargo lambda watch --invoke-port 3000

build:
	cargo lambda build

patch:
	cargo set-version --bump patch

clear-local-storage:
	rm -rf ./target/db
	rm -rf ./target/storage/unpkg

export-bindings:
	cargo run --example export_bindings
	rm -rf ../bevyhub-site/packages/shared/src/api-bindings/generated
	mkdir -p ../bevyhub-site/packages/shared/src/api-bindings/generated || true
	cp ./bindings/* ../bevyhub-site/packages/shared/src/api-bindings/generated


# 1. test
# 2. build the lambda
# 3. deploy the lambda
#   --verbose = enable tracing
# 4. reset prod db & s3
deploy *args:
	just test
	cargo lambda build --release
	cargo lambda deploy \
	--binary-name bevyhub_api \
	--iam-role arn:aws:iam::898915787211:role/bevyhub-lambda \
	--enable-function-url \
	--region {{region}} \
	--verbose \
	{{args}}
	just mongo-purge-prod

# cargo test --test bevyhub-api -- {{args}}

watch *command:
	forky watch \
	-w '**/*.rs' \
	-i '{.git,target,html}/**' \
	-i '**/mod.rs' \
	-- {{command}}

# you should rarely need to run this, usually mongo-purge-prod is all thats needed
purge-prod:
	just s3-purge-prod
	just mongo-purge-prod


mongosh-find *args:
	mongosh $MONGODB_CLIENT --eval "use db_prod" --eval "db.scenes.find({{args}})"

mongosh *args:
	mongosh $MONGODB_CLIENT

mongo-purge-prod:
	mongosh $MONGODB_CLIENT --eval "use db_prod" --eval  "db.dropDatabase()"

s3-purge-dev:
	aws s3 rm s3://bevyhub-dev --recursive

# view buckets here https://ap-southeast-2.console.aws.amazon.com/s3/home?region=ap-southeast-2#
s3-purge-prod:
	aws s3 rm s3://bevyhub-prod --recursive


s3-set-cors:
	aws s3api put-bucket-cors --bucket bevyhub-dev --cors-configuration file://./config/cors.json
	aws s3api put-bucket-cors --bucket bevyhub-prod --cors-configuration file://./config/cors.json


lambda-env:
	echo $MONGODB_CLIENT
	aws lambda update-function-configuration \
	--function-name bevyhub_api \
	--region {{region}}
	--environment "Variables={MONGODB_CLIENT=$MONGODB_CLIENT}" \

pws *args:
	just --shell powershell.exe --shell-arg -c {{args}}

curl *args:
	cargo run --example curl {{args}}
curl-prod *args:
	API_ENV=prod just curl


populate *args:
	just clear-local-storage
	just cli populate {{args}} \
	../bevyhub \
	../bevyhub/crates/bevyhub_template \
	../bevyhub/crates/bevyhub_net \
# ../beet \
# ../../me-temp/mrchantey_bevyhub_hello_world \
# ../sewb \
# ../beet/crates/beet_flow/ \
# ../beet/crates/beet_ml/ \
# ../beet/crates/beet_spatial/ \
# ../beet/crates/beet_examples/
