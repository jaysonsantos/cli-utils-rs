# cli-utils
I will store some utilities that I used on day to day and I chose rust to be able to use the same utilities in unix and windows systems.

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
