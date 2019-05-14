# cli-utils
I will store some utilities that I used on day to day and I chose rust to be able to use the same utilities in unix and windows systems.

# Installing the commands
To install you will need cargo installed.
To install all binaries run:
```bash
cargo install \
    --git https://github.com/jaysonsantos/cli-utils-rs
```
To install a specific binary:
```bash
cargo install \
    --git https://github.com/jaysonsantos/cli-utils-rs \
    --bin aws-ssm-env-importer
```

## Available commands
### delete-line
[delete-line](src/delete_line.rs) simply delete a line in a file.
```
delete-line 0.1.0

USAGE:
    delete-line <file> <line_number>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <file>           
    <line_number>    Line number (1) or range (1-10)
```
### delete-local-branches
[delete-local-branches](src/delete_local_branches.rs) delete all local branches except current active branch in order to
declutter

### aws-flow-logs
[aws-flow-logs](src/aws_flow_logs.rs) parse aws flow logs in a bucket.
```none
error: The following required arguments were not provided:
    <region>
    <bucket>
    <prefix>
    <filter_query>

USAGE:
    aws-flow-logs <region> <bucket> <prefix> <filter_query>
```
#### Examples
```none
aws-flow-logs eu-central-1 bucket prefix/2019/04/24 'src.port in {80 443} && dstport == 55540 && dstip in {10.0.0.0/8} && action == "REJECT"'
Matched with FlowLogLine {
    version: "2",
    account_id: "x",
    interface_id: "eni-x",
    srcaddr: V4(
        127.0.0.1,
    ),
    dstaddr: V4(
        10.0.145.125,
    ),
    srcport: 443,
    dstport: 55540,
    protocol: "6",
    packets: "1",
    bytes: 40,
    start: 1556114469,
    end: 1556114525,
    action: "REJECT",
    log_status: "OK",
}
```

### aws-ssm-env-importer
Import .env files into ssm using a template for the key.

```
USAGE:
    aws-ssm-env-importer [FLAGS] --app-name <app_name> --env-file <env_file> --environment <environment> --region <region> --template <template>

FLAGS:
    -d, --dry-run
    -h, --help         Prints help information
    -o, --overwrite
    -u, --uppercase
    -V, --version      Prints version information

OPTIONS:
    -a, --app-name <app_name>
    -f, --env-file <env_file>
    -e, --environment <environment>
    -r, --region <region>
    -t, --template <template>          Template to generate the key on SSM side, example
                                       "/{environment}/{app_name}/{key}"
```

#### Example
```bash
aws-ssm-env-importer \
    --env-file .env \
    --environment environment \
    --app-name test-app \
    --template "/{environment}/{app_name}/{key}" \
    --region eu-central-1 \
    --dry-run
 ```
