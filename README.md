# Runit
Run it like no tomorrow.

## Usage
First you need to create a `.runit` file in the root of your project.
The `.runit` file is a simple text file that contains a list of commands and an identifier.

Here is an example:
```runit
#!(identifier_name)
all code here
```

The first line has to be an identifier.
An identifier is similar to like a shebang if you want to think of it that way.
### Examples
Here we are executing a python script
```runit
#!python
print("Hello world")
import time
time.sleep(5)
print("Done!")
```

Now we are executing a shell script
```runit
#!shell
echo "Hello world"
sleep 5
echo "Done!"
```

You can also run docker containers (Requires sudo)
```runit
#!docker
FROM alpine:latest
RUN echo "Hello from Docker!"
RUN echo "Current directory contents:"
RUN ls -la
RUN echo "Docker test complete!"
```
