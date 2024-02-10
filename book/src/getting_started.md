# Getting Started
`jawk` is a command line tool thatshould be able to instract with other command line tools.
 
## Inputs
By default, `jawk` will read the input from the standart input (which can be piped from a previous command), but one can specify a file or directory in the command line, in that case, `jawk` will read from those files (if one of them is a directrory, `jawk` will read from all the readable files under that directory).
The inputs should be JSON values (objects, arrays, strings, numbers, Booleans and null). By default, if part of the input is not a valid JSON value, `jawk` will ignore it and will procceed to the next value.
`jawk` will then process each value on it's own, and will omit output for each value.

### Example
Running:
```
echo '
100
{"key-1": true, "key-2": false}    
"string"
nop
[null, 1, 2, 3]

' | jawk
```
Will return:
```
100
{"key-1": true, "key-2": false}
"string"
[null, 1, 2, 3]
```
## Selection
One can specify a few selection to perform on each value using the `--select` argument. See more details in the [selection](selection.md).
### Example
Running:
```
echo '
100
{"key-1": true, "key-2": false}    
"string"
nop
[null, 1, 2, 3]

' | jawk --select '(.len)=len'
```
Will return:
```
{}
{"len": 2}
{"len": 6}
{"len": 4}
```

## Output
By default the output will be a single line JSON for each valid value in the input. On can change this to CSV, text, or other styles of JSON. For more details see [the command line help](help.md).
### Example
Running:
```
echo '
{"key-1": true, "key-2": false}    
"string" 
100
nop
[null, 1, 2, 3]

' | jawk --select '(.len)=len' -o csv
```
Will return:
```
"len"
2
6

4
```
