To run these tests, you might come across system-compatibility bugs


Since this project is focused on x86 architecture, the assembly will be x86_64 and we will compile as x86_64.

If you own a different architecture system (for instance ARM64, like my MacBook Pro M2), then you might need to install and setup Rosetta

Install it once:
```bash
softwareupdate --install-rosetta --agree-to-license
```

and compile as such :
```bash 
clang -arch x86_64 assembler/tests/hello_x86.s -o assembler/output/hello_x86
arch -x86_64 ./assembler/output/hello_test
```

else if you want to fork and write ARM64, then it might look like this :

```bash
clang assembler/tests/hello_arm64.s -o prog
./prog
```
I might do a featureupdate regarding ARM64 later

-- Will
