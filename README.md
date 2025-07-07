# Run
Run it like no tomorrow.

Run is a dead simple project running utility.

## Usage
First you need to create a `.run` file in the root of your project.
The `.run` file is a simple text file that contains a list of commands and an identifier.

Here is an example:
```run
#!(identifier_name)
all code here
```

The first line has to be an identifier.
An identifier is similar to like a shebang if you want to think of it that way.

### Examples
Here we are executing a python script
```run
#!python
print("Hello world")
import time
time.sleep(5)
print("Done!")
```

Now we are executing a shell script (Decided by $SHELL environment variable)
```run
#!shell
echo "Hello world"
sleep 5
echo "Done!"
```

You can also run docker containers (Requires sudo)
```run
#!docker
FROM alpine:latest
RUN echo "Hello from Docker!"
RUN echo "Current directory contents:"
RUN ls -la
RUN echo "Docker test complete!"
```

And you can also run Ruby code too
```run
#!ruby
puts "Hello world"
```

## Useful errors

```run
!rubby
puts "Hello world"
```

```error
Error: Error::Syntax

  × Syntax error
   ╭─[./tests/2ruby:1:1]
 1 │ !rubby
   · ───┬──
   ·    ╰── Unexpected identifier
   ╰────
  help: Did you mean? #!ruby
```

## Installation
```bash
git clone https://github.com/stewthestew/run.git
cd run
cargo install --path .
```
