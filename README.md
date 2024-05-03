[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/iYoQzOhX)
# Rustybox

**echo**:
Echo is very simple. What the function does is that checks whether the newline 
option "-n" is active and that will remove the newline at the end of the output. 
Then it prints for every words set for the function. Finally, if the "-n" option 
was not set, the function will produce a newline with a return value of 0.

**mkdir**:
Mkdir was 7 lines of code so not much to explain. Basically what this function 
does is to create a directory, if it succeeded, then it will return 0, if not, 
then the specified error will show with a returned number.

**cat**:
Same story, but even fewer lines. This function uses the "read_to_string" 
function to read the content of a file, then it prints it. Simple.

**pwd**:
Same format as the previous discussed function. This one returns the current 
working directory in the "current_dir" variable and it prints it out.

**mv**:
Now a teaser for the more complicated functions to describe. The "**mv**" 
function which is done with a function named "rename" and if it fails, then, 
there is a handler to copy the source file to the destination and then remove 
the source file. The copy_and_remove function takes only two arguments, source 
and destination, which are strings representing paths.

**ln**:
The "ln" function takes 3 arguments: the source file, the desired name for the 
link and a boolean indicating whether the link should be symbolic or hard. This 
function checks if the source path exists or not. Then it checks if the link 
already exists. And then, it makes the link based on what linkage needed.
And there is one more step, to check if the linking process was a success.

**ls**:
This is a bigger function than what I've just describe earlier so bare with me.
This one checks if the path doesn't exist, so it prints an error message with 
the desired error for the return. After that, it checks if the path is a file 
and not a directory. Then we have to search for a recursive option, if it exists 
of course, to call the helper function "ls_recursive", but we'll get into that 
after this function. If the command isn't meant to be a recursive search, then 
all it must do is to print the contents of the directory. We go through every 
filename and prints the required files, whether the "-a" option is put or not.

And now for the recursive one. It does the same thing. Only recursively :)

**rm**:
What it does is to simply check for options in the first place and then iterate
 through the arguments for directories or files and to check the cases for 
errors. At the very end, if the an error is triggered, it will print a message 
and return an error....too much "errors" said here.

**grep**:
Didn't quite understand why I had errors at the echo function at tests with the 
same name. I thought it was a simple mistake, but no....it was just a happy 
accident :) .

So what it does is to iterate through the filenames and to check if the lines 
themselves contain some pattern. If they do, then to print them.

**cp**:
Now this function has a helper function which does the job. It simply checks if 
there is a directory (if there is not, then it will create a directory) and, 
later, checks if the call is recursive, if it is, then it will just re-call 
itself. If the destination isn't a directory, then it will copy a file. And now 
for the main function. Firstly, it checks if the source already exists, then it 
checks for the case where the destination is a directory, which in this case, 
must be at the end, with the name of a file. Then to check if the destination 
doesn't exist. Then it will solve the case where we want to copy a file or to 
copy a whole directory recursively, it will call the helper function.

**rmdir**:
This function is something a little bit easier. This one checks if the path even
exists, then it checks if the path is a directory or a file. Then it removes 
whatever it is.

**chmod**
Now for a big one. This function is no joke. It was very complicated. Firstly, 
it checks if the input string is a valid octal mode, then by each case goes to
its own destination. Let's talk about "octal permissions" first. It changes the
mode from a string to an int using "from_str_radix". Then, after it has succedeed, 
it will set the permissions to the file using "fs::set_permissions" function.
The first part is done....now the hard one. The "set_symbolic_permissions" function
takes the metadata of the file. If the metadata could not be taken, the function 
will return an error. The function extracts the current permissions of the file
or directory from that metadata. Then, the mode is parsed character by character to
take the changes to be made. For each permission change, the function updates the 
current mode of the permissions based on the permission target, action, and 
permission character.  If the permissions cannot be set, the function returns an 
error.

**touch**
Finally...the last function. The touch function takes in a file path, and three
 boolean flags: change_access, no_create, and change_modify. It then checks if the 
file exists, and if it does, it changes the access and/or modification time of the 
file based on the boolean flags. If the file does not exist and no_create is false, 
it creates the file. The function returns an io::Result indicating whether the 
operation was successful or not.

## Verify

Run the following commands to test your homework:

You will have to install NodeJS (it is installed in the codespace)

```bash
# Clone tests repository
git submodule update --init 

# Update tests repository to the lastest version
cd tests
git pull 
cd ..

# Install loadash
npm install lodash
```

Install rustybox

```bash
cargo install --path .
```

Run tests

```bash
cd tests
# Run all tests 
./run_all.sh

# Run single test
./run_all.sh pwd/pwd.sh
```# rustybox
