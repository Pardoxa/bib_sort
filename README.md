# For sorting bib files (LaTeX)

The idea is: You have bib files containing items like 

```bib
@article{boers2019,
    author = {N. Boers AND B. Goswami AND A. Rheinwalt AND B. Bookhagen AND B. Hoskins AND J. Kurths},
    title = {Complex networks reveal global pattern of extreme-rainfall teleconnections},
    journal = {Nature},
    year = 2019,
    volume = {566},
    pages = {373-377},
    doi = {10.1038/s41586-018-0872-x}
}

@Book{baxter1982,
  author =    {R. J. Baxter},
  title =     {Exactly Solved Models in Statistical Mechanics},
  publisher = {Academic Press},
  address =   {London},
  year =      {1982},
}
```

and the bib item should be sorted according to the key (here boers2019 and baxter1982).
By default the sorting is case insensitive.

It will print the sorted files to the terminal

## Compiling

If you do not have Rust (and cargo) installed, install it [https://doc.rust-lang.org/book/ch01-01-installation.html].
Then clone this repository. Open a terminal in the folder of this README.md

### Option 1)
Compile via:
```bash
cargo b -r
```
The executable "bib_sort" is now in the folder ./target/release
You need to add the executable to your PATH to be able to call it as bib_sort from anywhere.

### Option 2)
You can also run
```bash
cargo install --path .
```
when inside the folder of this README.md 
This will add the executable to .cargo/bin which is 
added to your PATH during the installation of Rust, 
at least with default settings, so you can skip the PATH adding 
step and still call bib_sort from anywhere

# Usage

Bibfiles are expected to be encoded in UTF8. ASCII is also fine.
Other encodings may lead to errors.

Sorting bibfiles:
```bash
bib_sort path.bib
```
will print the sorted file in the terminal

```bash
bib_sort path.bib -o out.bib
```
will write the sorted bibfile to the file out.bib

```bash
bib_sort --help
```
for help Info

Note: If you want to overwrite the bibfile: Do **NOT** pipe into it. 
Commands like "bib_sort literature.bib > literature.bib" will **DELETE** the literature.bib file 
before the program reads it, which means it is essentially just creating an empty file.

**INSTEAD**: You can safely use "bib_sort literature.bib -o literature.bib" as this will first parse
the bibfile and then overwrite it with the sorted file only if no errors were detected.